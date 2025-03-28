use crate::errors::EndpointManagerError;
use crate::session::get_session;
use crate::utils::{CongestionControl, HandlerChannel, Priority};
use once_cell::sync::OnceCell;
use prost::Message;
use serde::Deserialize;
use std::clone::Clone;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::time::timeout as tokio_timeout;
use zenoh::bytes::{Encoding, ZBytes};
use zenoh::key_expr::KeyExpr;
use zenoh::liveliness::LivelinessToken;
use zenoh::query::{Query, Queryable, Selector};
use zenoh::sample::SampleKind;
use zenoh::{qos, Session, Wait};

#[derive(Deserialize, Clone)]
#[serde(tag = "endpoint_type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Endpoint {
    Req {
        endpoint_name: String,
        endpoint_key: String,
        requester_message_type: String,
        provider_message_type: String,
        congestion_control: Option<CongestionControl>,
        priority: Option<Priority>,
        express: Option<bool>,
    },
    Prv {
        endpoint_name: String,
        endpoint_key: String,
        requester_message_type: String,
        provider_message_type: String,
        handler: Option<HandlerChannel>,
    },
}

#[derive(Deserialize, Clone)]
struct Endpoints {
    endpoints: Vec<Endpoint>,
}

struct EndpointManager {
    session: Arc<Session>,
    endpoints: RwLock<HashMap<String, Arc<EndpointType>>>,
    endpoint_names: RwLock<HashMap<String, String>>,
}

impl EndpointManager {
    fn initialize() -> Result<Self, EndpointManagerError> {
        let session = get_session();

        let endpoint_data = parse_endpoints()?; // returns Endpoints
        let mut endpoints_map = HashMap::new();
        let mut endpoint_names_map = HashMap::new();

        for endpoint in endpoint_data.endpoints {
            match endpoint {
                Endpoint::Req {
                    endpoint_name,
                    endpoint_key,
                    congestion_control,
                    priority,
                    express,
                    ..
                } => {
                    let qos_priority = priority.unwrap_or(Priority::Data).to_zenoh();
                    let qos_congestion = congestion_control
                        .unwrap_or(CongestionControl::Block)
                        .to_zenoh();
                    let express = express.unwrap_or(true);

                    let requester = Requester::new(
                        &endpoint_key,
                        session.clone(),
                        qos_priority,
                        express,
                        qos_congestion,
                    )?;

                    endpoints_map.insert(
                        endpoint_key.clone(),
                        Arc::new(EndpointType::Requester(Arc::new(requester))),
                    );
                    endpoint_names_map.insert(endpoint_name.clone(), endpoint_key.clone());
                }

                Endpoint::Prv {
                    endpoint_name,
                    endpoint_key,
                    handler,
                    ..
                } => {
                    let handler = handler.unwrap_or(HandlerChannel::Fifo {
                        capacity: Some(100),
                    });

                    let provider = Provider::new(&endpoint_key, session.clone(), handler)?;
                    endpoints_map.insert(
                        endpoint_key.clone(),
                        Arc::new(EndpointType::Provider(Arc::new(provider))),
                    );
                    endpoint_names_map.insert(endpoint_name.clone(), endpoint_key.clone());
                }
            }
        }

        Ok(EndpointManager {
            session,
            endpoints: RwLock::new(endpoints_map),
            endpoint_names: RwLock::new(endpoint_names_map),
        })
    }

    fn get_requester<TReq, TRes>(&self, name: &str) -> Option<TypedRequester<TReq, TRes>>
    where
        TReq: Message + Default,
        TRes: Message + Default,
    {
        let endpoints_read = self.endpoints.read().ok()?;
        let endpoint_arc = endpoints_read.get(name)?.clone();
        match &*endpoint_arc {
            EndpointType::Requester(requester_endpoint) => Some(TypedRequester {
                inner: Arc::clone(requester_endpoint),
                _phantom: (PhantomData, PhantomData),
            }),
            _ => None,
        }
    }

    fn get_provider<TReq, TRes>(&self, name: &str) -> Option<TypedProvider<TReq, TRes>>
    where
        TReq: Message + Default,
        TRes: Message + Default,
    {
        let endpoints_read = self.endpoints.read().ok()?;
        let endpoint_arc = endpoints_read.get(name)?.clone();
        match &*endpoint_arc {
            EndpointType::Provider(provider_endpoint) => Some(TypedProvider {
                inner: Arc::clone(provider_endpoint),
                _phantom: (PhantomData, PhantomData),
            }),
            _ => None,
        }
    }

    fn resolve_endpoint_name(&self, name: &str) -> Option<String> {
        self.endpoint_names.read().ok()?.get(name).cloned()
    }
}

enum EndpointType {
    Requester(Arc<Requester>),
    Provider(Arc<Provider>),
}

static ENDPOINT_MANAGER: OnceCell<EndpointManager> = OnceCell::new();

fn parse_endpoints() -> Result<Endpoints, EndpointManagerError> {
    let env = std::env::var("ENDPOINTS")?;
    let endpoints = serde_json::from_str(&env)?;
    Ok(endpoints)
}

pub fn resolve_endpoint_name(name: &str) -> Option<String> {
    ENDPOINT_MANAGER
        .get()
        .and_then(|manager| manager.resolve_endpoint_name(&name))
}

pub fn get_requester<TReq, TRes>(name: String) -> Option<TypedRequester<TReq, TRes>>
where
    TReq: Message + Default,
    TRes: Message + Default,
{
    ENDPOINT_MANAGER
        .get()
        .and_then(|manager| manager.get_requester(&name))
}

pub fn get_provider<TReq, TRes>(name: String) -> Option<TypedProvider<TReq, TRes>>
where
    TReq: Message + Default,
    TRes: Message + Default,
{
    ENDPOINT_MANAGER
        .get()
        .and_then(|manager| manager.get_provider(&name))
}

pub struct TypedProvider<TReq, TRes> {
    inner: Arc<Provider>,
    _phantom: (PhantomData<TReq>, PhantomData<TRes>),
}

impl<TReq, TRes> TypedProvider<TReq, TRes>
where
    TReq: Message + Default,
    TRes: Message + Default,
{
    pub fn provide<F>(&self, callback: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(TReq) -> TRes + Send + Sync + 'static,
    {
        let key_expr = self.inner.key_expr.clone();

        self.inner.provide(move |query| {
            let empty_bytes = ZBytes::default();
            let bytes = query.payload().unwrap_or_else(|| &empty_bytes).to_bytes();
            let message = TReq::decode(&*bytes).unwrap();
            let response = callback(message);
            match query
                .reply(key_expr.clone(), ZBytes::from(response.encode_to_vec()))
                .encoding(Encoding::APPLICATION_PROTOBUF)
                .priority(qos::Priority::RealTime)
                .express(true)
                .congestion_control(qos::CongestionControl::Block)
                .wait()
            {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error sending response: {:?}", e);
                }
            }
        })?;
        Ok(())
    }

    pub async fn provide_async<F, Fut>(
        &self,
        callback: F,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(TReq) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = TRes> + Send + 'static,
    {
        let key_expr = self.inner.key_expr.clone();
        let callback = Arc::new(callback);

        self.inner
            .provide_async(move |query| {
                let callback = Arc::clone(&callback);
                let key_expr = key_expr.clone();
                async move {
                    let empty_bytes = ZBytes::default();
                    let bytes = query.payload().unwrap_or(&empty_bytes).to_bytes();
                    let message = TReq::decode(&*bytes).unwrap();

                    let response = callback(message).await;

                    if let Err(e) = query
                        .reply(key_expr.clone(), ZBytes::from(response.encode_to_vec()))
                        .encoding(Encoding::APPLICATION_PROTOBUF)
                        .priority(qos::Priority::RealTime)
                        .express(true)
                        .congestion_control(qos::CongestionControl::Block)
                        .await
                    {
                        eprintln!("Error sending response: {:?}", e);
                    }
                }
            })
            .await?;
        Ok(())
    }
}

pub struct TypedRequester<TReq, TRes> {
    inner: Arc<Requester>,
    _phantom: (PhantomData<TReq>, PhantomData<TRes>),
}
impl<TReq, TRes> TypedRequester<TReq, TRes>
where
    TReq: Message + Default,
    TRes: Message + Default,
{
    pub fn request(
        &self,
        message: &TReq,
        timeout: Option<Duration>,
    ) -> Result<TRes, Box<dyn Error + Send + Sync>> {
        let subscriber = self
            .inner
            .session
            .liveliness()
            .declare_subscriber(&self.inner.key_expr)
            .history(true)
            .wait()?;

        let sample = match timeout {
            Some(timeout) => subscriber.recv_timeout(timeout),
            None => subscriber.recv().map(|s| Some(s)),
        }
        .map_err(|_| EndpointManagerError::EndpointNotAvailable(self.inner.name.clone()))?
        .ok_or_else(|| EndpointManagerError::EndpointNotAvailable(self.inner.name.clone()))?;

        if sample.kind() != SampleKind::Put {
            return Err(Box::new(EndpointManagerError::EndpointNotAvailable(
                self.inner.name.clone(),
            )));
        }

        let reply = self
            .inner
            .session
            .get(&self.inner.selector)
            .payload(ZBytes::from(message.encode_to_vec()))
            .encoding(Encoding::APPLICATION_PROTOBUF)
            .priority(self.inner.priority)
            .express(self.inner.express)
            .congestion_control(self.inner.congestion_control)
            .wait()?
            .recv()?;
        let response_sample = reply
            .result()
            .map_err(|_| EndpointManagerError::EndpointNotAvailable(self.inner.name.clone()))?;

        let bytes = response_sample.payload().to_bytes();
        TRes::decode(&*bytes).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    pub async fn request_async(
        &self,
        message: &TReq,
        timeout: Option<Duration>,
    ) -> Result<TRes, Box<dyn Error + Send + Sync>> {
        let selector = self.inner.selector.clone();
        let payload = ZBytes::from(message.encode_to_vec());

        let query = self
            .inner
            .session
            .get(&selector)
            .payload(payload)
            .encoding(Encoding::APPLICATION_PROTOBUF)
            .priority(self.inner.priority)
            .express(self.inner.express)
            .congestion_control(self.inner.congestion_control)
            .await?;

        let reply_future = query.recv_async();

        let reply_sample = match timeout {
            Some(duration) => {
                match tokio_timeout(duration, reply_future).await {
                    Ok(Ok(sample)) => sample,
                    Ok(Err(_)) => {
                        return Err(Box::new(EndpointManagerError::EndpointNotAvailable(
                            self.inner.name.clone(),
                        )));
                    }
                    Err(_) => {
                        // Timeout occurred
                        return Err(Box::new(EndpointManagerError::EndpointNotAvailable(
                            self.inner.name.clone(),
                        )));
                    }
                }
            }
            None => reply_future.await?,
        };

        let result = reply_sample.into_result()?;
        let bytes = result.payload().to_bytes();
        let response = TRes::decode(&*bytes)?;
        Ok(response)
    }
}

struct Requester {
    session: Arc<Session>,
    name: String,
    selector: Selector<'static>,
    key_expr: KeyExpr<'static>,
    priority: qos::Priority,
    express: bool,
    congestion_control: qos::CongestionControl,
}

impl Requester {
    pub fn new(
        name: &str,
        session: Arc<Session>,
        priority: qos::Priority,
        express: bool,
        congestion_control: qos::CongestionControl,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let key_expr = KeyExpr::autocanonize(name.to_string())?;
        Ok(Requester {
            session: Arc::clone(&session),
            name: name.to_string(),
            selector: Selector::from(key_expr.clone()),
            key_expr,
            priority,
            express,
            congestion_control,
        })
    }
}

struct Provider {
    session: Arc<Session>,
    name: String,
    key_expr: KeyExpr<'static>,
    queryable: Mutex<Option<Queryable<()>>>,
    token: Mutex<Option<LivelinessToken>>,
}

impl Provider {
    pub fn new(
        name: &str,
        session: Arc<Session>,
        handler: HandlerChannel,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // TODO: Implement handler-based provider
        Ok(Provider {
            session: Arc::clone(&session),
            name: name.to_string(),
            key_expr: KeyExpr::autocanonize(name.to_string())?,
            queryable: Mutex::new(None),
            token: Mutex::new(None),
        })
    }

    pub fn provide<F>(&self, callback: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(Query) + Send + Sync + 'static,
    {
        let queryable = self
            .session
            .declare_queryable(&self.key_expr)
            .callback(callback)
            .wait()?;

        {
            let mut queryable_lock = self.queryable.lock().unwrap();
            *queryable_lock = Some(queryable);
        }

        let token = self
            .session
            .liveliness()
            .declare_token(&self.key_expr)
            .wait()?;

        {
            let mut token_lock = self.token.lock().unwrap();
            *token_lock = Some(token);
        }

        Ok(())
    }

    pub async fn provide_async<F, Fut>(
        &self,
        callback: F,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(Query) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let handle = Handle::current();
        let session = Arc::clone(&self.session);
        let key_expr = self.key_expr.clone();

        let queryable = session
            .declare_queryable(&key_expr)
            .callback(move |query| {
                let fut = callback(query);
                handle.spawn(fut);
            })
            .await?;

        {
            let mut queryable_lock = self.queryable.lock().unwrap();
            *queryable_lock = Some(queryable);
        }

        let token = session.liveliness().declare_token(&key_expr).await?;

        {
            let mut token_lock = self.token.lock().unwrap();
            *token_lock = Some(token);
        }

        Ok(())
    }
}

pub(crate) fn initialize() -> Result<(), Box<dyn Error + Send + Sync>> {
    let manager = EndpointManager::initialize()?;
    ENDPOINT_MANAGER
        .set(manager)
        .map_err(|_| "EndpointManager is already initialized")?;
    Ok(())
}
