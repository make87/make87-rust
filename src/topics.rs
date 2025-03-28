use crate::errors::TopicManagerError;
use crate::session::get_session;
use once_cell::sync::OnceCell;
use prost::Message;
use serde::Deserialize;
use std::any::type_name;
use std::clone::Clone;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use tokio::runtime::Handle;
use zenoh::bytes::{Encoding, ZBytes};
use zenoh::handlers::{FifoChannel, FifoChannelHandler, RingChannel, RingChannelHandler};
use zenoh::key_expr::KeyExpr;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::pubsub::Subscriber as ZenohSubscriber;
use zenoh::sample::Sample;
use zenoh::{qos, Session, Wait};
use crate::utils::{CongestionControl, HandlerChannel, Priority, Reliability};

#[derive(Deserialize, Clone)]
struct Topics {
    topics: Vec<Topic>,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "topic_type")]
enum Topic {
    PUB {
        topic_name: String,
        topic_key: String,
        message_type: String,
        congestion_control: Option<CongestionControl>,
        priority: Option<Priority>,
        express: Option<bool>,
        reliability: Option<Reliability>,
    },
    SUB {
        topic_name: String,
        topic_key: String,
        message_type: String,
        handler: Option<HandlerChannel>,
    },
}

struct TopicManager {
    session: Arc<Session>,
    topics: RwLock<HashMap<String, Arc<TopicType>>>,
    topic_names: RwLock<HashMap<String, String>>,
}

impl TopicManager {
    fn initialize() -> Result<Self, TopicManagerError> {
        let session = get_session();

        let topic_data = parse_topics()?; // returns Topics
        let mut topics_map = HashMap::new();
        let mut topic_names_map = HashMap::new();

        for topic in topic_data.topics {
            match topic {
                Topic::PUB {
                    topic_name,
                    topic_key,
                    congestion_control,
                    priority,
                    express,
                    reliability,
                    ..
                } => {
                    let qos_priority = priority.unwrap_or(Priority::Data).to_zenoh();
                    let qos_reliability = reliability.unwrap_or(Reliability::Reliable).to_zenoh();
                    let qos_congestion = congestion_control
                        .unwrap_or(CongestionControl::Drop)
                        .to_zenoh();
                    let express = express.unwrap_or(true);

                    let publisher = Publisher::new(
                        &topic_key,
                        session.clone(),
                        qos_priority,
                        express,
                        qos_reliability,
                        qos_congestion,
                    )?;

                    topics_map.insert(topic_key.clone(), Arc::new(TopicType::Publisher(Arc::new(publisher))));
                    topic_names_map.insert(topic_name.clone(), topic_key.clone());
                }

                Topic::SUB {
                    topic_name,
                    topic_key,
                    handler,
                    ..
                } => {
                    let handler = handler.unwrap_or(HandlerChannel::Ring {
                        capacity: Some(100)
                    });

                    let subscriber = Subscriber::new(&topic_key, session.clone(), handler)?;
                    topics_map.insert(topic_key.clone(), Arc::new(TopicType::Subscriber(Arc::new(subscriber))));
                    topic_names_map.insert(topic_name.clone(), topic_key.clone());
                }
            }
        }

        Ok(TopicManager {
            session,
            topics: RwLock::new(topics_map),
            topic_names: RwLock::new(topic_names_map),
        })
    }

    fn get_publisher<T>(&self, name: &str) -> Option<TypedPublisher<T>>
    where
        T: Message + Default,
    {
        let topics_read = self.topics.read().ok()?;
        let topic_arc = topics_read.get(name)?.clone();
        match &*topic_arc {
            TopicType::Publisher(publisher_topic) => Some(TypedPublisher {
                inner: Arc::clone(publisher_topic),
                _phantom: PhantomData,
            }),
            _ => None,
        }
    }

    fn get_subscriber<T>(&self, name: &str) -> Option<TypedSubscriber<T>>
    where
        T: Message + Default,
    {
        let topics_read = self.topics.read().ok()?;
        let topic_arc = topics_read.get(name)?.clone();
        match &*topic_arc {
            TopicType::Subscriber(subscriber_topic) => Some(TypedSubscriber {
                inner: Arc::clone(subscriber_topic),
                _phantom: PhantomData,
            }),
            _ => None,
        }
    }

    fn resolve_topic_name(&self, name: &str) -> Option<String> {
        self.topic_names.read().ok()?.get(name).cloned()
    }
}

enum TopicType {
    Publisher(Arc<Publisher>),
    Subscriber(Arc<Subscriber>),
}

static TOPIC_MANAGER: OnceCell<TopicManager> = OnceCell::new();

fn parse_topics() -> Result<Topics, TopicManagerError> {
    let env = std::env::var("TOPICS")?;
    let topics = serde_json::from_str(&env)?;
    Ok(topics)
}

pub fn resolve_topic_name(name: &str) -> Option<String> {
    TOPIC_MANAGER
        .get()
        .and_then(|manager| manager.resolve_topic_name(&name))
}

pub fn get_publisher<T>(name: String) -> Option<TypedPublisher<T>>
where
    T: Message + Default,
{
    TOPIC_MANAGER
        .get()
        .and_then(|manager| manager.get_publisher(&name))
}

pub fn get_subscriber<T>(name: String) -> Option<TypedSubscriber<T>>
where
    T: Message + Default,
{
    TOPIC_MANAGER
        .get()
        .and_then(|manager| manager.get_subscriber(&name))
}

#[derive(Debug)]
pub struct Metadata {
    pub topic_name: String,
    pub message_type_decoded: String,
    pub bytes_transmitted: usize,
}

pub struct MessageWithMetadata<T> {
    pub message: T,
    pub metadata: Metadata,
}

pub struct TypedSubscriber<T> {
    inner: Arc<Subscriber>,
    _phantom: PhantomData<T>,
}

impl<T> TypedSubscriber<T>
where
    T: Message + Default + 'static,
{
    pub fn receive(&self) -> Result<T, Box<dyn Error>> {
        let sample;
        match &self.inner.subscriber {
            SubscriberType::Fifo(sub) => {
                sample = sub.recv().unwrap();
            }
            SubscriberType::Ring(sub) => {
                sample = sub.recv().unwrap();
            }
        }
        let bytes = sample.payload().to_bytes();
        T::decode(&*bytes).map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    pub fn receive_with_metadata(&self) -> Result<MessageWithMetadata<T>, Box<dyn Error>> {
        let sample;
        match &self.inner.subscriber {
            SubscriberType::Fifo(sub) => {
                sample = sub.recv().unwrap();
            }
            SubscriberType::Ring(sub) => {
                sample = sub.recv().unwrap();
            }
        }
        let bytes = sample.payload().to_bytes();
        match T::decode(&*bytes) {
            Ok(message) => Ok(MessageWithMetadata {
                metadata: Metadata {
                    topic_name: sample.key_expr().to_string(),
                    message_type_decoded: type_name::<T>().to_string(),
                    bytes_transmitted: bytes.len(),
                },
                message,
            }),
            Err(e) => Err(Box::new(e) as Box<dyn Error>),
        }
    }

    pub fn subscribe<F>(&self, callback: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        self.inner.subscribe(move |sample| {
            let bytes = sample.payload().to_bytes();
            match T::decode(&*bytes) {
                Ok(message) => {
                    callback(message);
                }
                Err(e) => {
                    eprintln!("Failed to decode message: {:?}", e);
                }
            }
        })?;
        Ok(())
    }

    pub fn subscribe_with_metadata<F>(
        &self,
        callback: F,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(MessageWithMetadata<T>) + Send + Sync + 'static,
    {
        self.inner.subscribe(move |sample| {
            let bytes = sample.payload().to_bytes();
            match T::decode(&*bytes) {
                Ok(message) => {
                    let metadata = Metadata {
                        topic_name: sample.key_expr().to_string(),
                        message_type_decoded: type_name::<T>().to_string(),
                        bytes_transmitted: bytes.len(),
                    };
                    callback(MessageWithMetadata { message, metadata })
                }
                Err(e) => {
                    eprintln!("Failed to decode message: {:?}", e);
                }
            }
        })?;
        Ok(())
    }

    pub async fn receive_async(&self) -> Result<T, Box<dyn Error + Send + Sync>> {
        let sample;
        match &self.inner.subscriber {
            SubscriberType::Fifo(sub) => {
                sample = sub.recv_async().await?;
            }
            SubscriberType::Ring(sub) => {
                sample = sub.recv_async().await?;
            }
        }
        let bytes = sample.payload().to_bytes();
        T::decode(&*bytes).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    pub async fn receive_with_metadata_async(
        &self,
    ) -> Result<MessageWithMetadata<T>, Box<dyn Error + Send + Sync>> {
        let sample;
        match &self.inner.subscriber {
            SubscriberType::Fifo(sub) => {
                sample = sub.recv_async().await?;
            }
            SubscriberType::Ring(sub) => {
                sample = sub.recv_async().await?;
            }
        }
        let bytes = sample.payload().to_bytes();
        match T::decode(&*bytes) {
            Ok(message) => Ok(MessageWithMetadata {
                metadata: Metadata {
                    topic_name: sample.key_expr().to_string(),
                    message_type_decoded: type_name::<T>().to_string(),
                    bytes_transmitted: bytes.len(),
                },
                message,
            }),
            Err(e) => Err(Box::new(e) as Box<dyn Error + Send + Sync>),
        }
    }

    pub async fn subscribe_async<F, Fut>(
        &self,
        callback: F,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output=()> + Send + 'static,
    {
        let callback = Arc::new(callback);
        self.inner.subscribe_async(move |sample| {
            let callback = Arc::clone(&callback);

            // Extract necessary data from sample
            let bytes = sample.payload().to_bytes();
            let message_result = T::decode(&*bytes);

            let fut = async move {
                match message_result {
                    Ok(message) => {
                        callback(message).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to decode message: {:?}", e);
                    }
                }
            };
            Handle::current().spawn(fut);
        })?;
        Ok(())
    }

    pub async fn subscribe_with_metadata_async<F, Fut>(
        &self,
        callback: F,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(MessageWithMetadata<T>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output=()> + Send + 'static,
    {
        let callback = Arc::new(callback);
        self.inner.subscribe_async(move |sample| {
            let callback = Arc::clone(&callback);

            // Extract necessary data from sample
            let bytes = sample.payload().to_bytes();
            let key_expr_string = sample.key_expr().to_string();
            let bytes_transmitted = bytes.len();
            let message_result = T::decode(&*bytes);

            let fut = async move {
                match message_result {
                    Ok(message) => {
                        let metadata = Metadata {
                            topic_name: key_expr_string,
                            message_type_decoded: type_name::<T>().to_string(),
                            bytes_transmitted,
                        };
                        callback(MessageWithMetadata { message, metadata }).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to decode message: {:?}", e);
                    }
                }
            };
            Handle::current().spawn(fut);
        })?;
        Ok(())
    }
}

pub struct TypedPublisher<T> {
    inner: Arc<Publisher>,
    _phantom: PhantomData<T>,
}

impl<T> TypedPublisher<T>
where
    T: Message + Default,
{
    pub fn publish(&self, message: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.inner
            .publisher
            .put(ZBytes::from(message.encode_to_vec()))
            .wait()?;
        Ok(())
    }

    pub async fn publish_async(&self, message: &T) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.inner
            .publisher
            .put(ZBytes::from(message.encode_to_vec()))
            .await?;
        Ok(())
    }
}

struct Publisher {
    name: String,
    publisher: ZenohPublisher<'static>,
}

impl Publisher {
    pub fn new(
        name: &str,
        session: Arc<Session>,
        priority: qos::Priority,
        express: bool,
        reliability: qos::Reliability,
        congestion_control: qos::CongestionControl,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let publisher = session
            .declare_publisher(name.to_string())
            .encoding(Encoding::APPLICATION_PROTOBUF)
            .priority(priority)
            .express(express)
            .reliability(reliability)
            .congestion_control(congestion_control)
            .wait()?;

        Ok(Publisher {
            name: name.to_string(),
            publisher,
        })
    }
}

enum SubscriberType {
    Fifo(ZenohSubscriber<FifoChannelHandler<Sample>>),
    Ring(ZenohSubscriber<RingChannelHandler<Sample>>),
}

struct Subscriber {
    session: Arc<Session>,
    name: String,
    subscriber: SubscriberType,
}

impl Subscriber {
    pub fn new(
        name: &str,
        session: Arc<Session>,
        handler: HandlerChannel,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let key_expr = KeyExpr::new(name)?;

        let subscriber = match handler {
            HandlerChannel::Fifo { capacity } => {
                let cap = capacity.unwrap_or(100);
                let fifo_handler = FifoChannel::new(cap);
                let sub = session
                    .declare_subscriber(&key_expr)
                    .with(fifo_handler)
                    .wait()?;
                SubscriberType::Fifo(sub)
            }
            HandlerChannel::Ring { capacity } => {
                let cap = capacity.unwrap_or(100);
                let ring_handler = RingChannel::new(cap);
                let sub = session
                    .declare_subscriber(&key_expr)
                    .with(ring_handler)
                    .wait()?;
                SubscriberType::Ring(sub)
            }
        };

        Ok(Subscriber {
            session,
            name: name.to_string(),
            subscriber,
        })
    }

    pub fn subscribe<F>(&self, callback: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(Sample) + Send + Sync + 'static,
    {
        self.session
            .declare_subscriber(KeyExpr::autocanonize(self.name.to_string())?)
            .callback(callback)
            .background()
            .wait()?;

        Ok(())
    }

    pub fn subscribe_async<F>(&self, callback: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(Sample) + Send + Sync + 'static,
    {
        self.session
            .declare_subscriber(KeyExpr::autocanonize(self.name.to_string())?)
            .callback(callback)
            .background()
            .wait()
            .unwrap();

        Ok(())
    }
}

pub(crate) fn initialize() -> Result<(), Box<dyn Error + Send + Sync>> {
    let manager = TopicManager::initialize()?;
    TOPIC_MANAGER
        .set(manager)
        .map_err(|_| "TopicManager is already initialized")?;
    Ok(())
}
