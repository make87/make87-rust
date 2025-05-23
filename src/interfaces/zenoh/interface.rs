use crate::config::load_config_from_default_env;
use crate::interfaces::zenoh::model::{HandlerChannel, ZenohProviderConfig, ZenohPublisherConfig, ZenohRequesterConfig, ZenohSubscriberConfig};
use crate::models::{ApplicationConfig, EndpointConfig, TopicConfig};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::error::Error as StdError;
use zenoh::handlers::{FifoChannel, FifoChannelHandler, RingChannel, RingChannelHandler};
use zenoh::pubsub::{Publisher, Subscriber};
use zenoh::query::{Querier, Query, Queryable};
use zenoh::sample::Sample;
use zenoh::Error as ZError;
use zenoh::{Config, Session};

fn decode_config<T: serde::de::DeserializeOwned>(map: &BTreeMap<String, Value>) -> Result<T, ZError> {
    Ok(serde_json::from_value(Value::Object(map.clone().into_iter().collect()))?)
}


#[derive(Debug, thiserror::Error)]
pub enum ZenohInterfaceError {
    #[error("No publisher topic found with name: {0}")]
    PubTopicNotFound(String),
    #[error("No subscriber topic found with name: {0}")]
    SubTopicNotFound(String),
    #[error("No requester endpoint found with name: {0}")]
    ReqEndpointNotFound(String),
    #[error("No provider endpoint found with name: {0}")]
    PrvEndpointNotFound(String),
    #[error(transparent)]
    Other(#[from] Box<dyn StdError + Send + Sync>),
}


pub enum ConfiguredSubscriber {
    Fifo(Subscriber<FifoChannelHandler<Sample>>),
    Ring(Subscriber<RingChannelHandler<Sample>>),
}

pub enum ConfiguredProvider {
    Fifo(Queryable<FifoChannelHandler<Query>>),
    Ring(Queryable<RingChannelHandler<Query>>),
}

pub struct ZenohInterface {
    config: ApplicationConfig,
}

impl ZenohInterface {
    pub fn new(config: ApplicationConfig) -> Self {
        Self { config }
    }

    pub fn from_default_env() -> Result<Self, Box<dyn StdError + Send + Sync>> {
        let config = load_config_from_default_env()?;
        Ok(Self { config })
    }

    pub fn zenoh_config(&self) -> Result<Config, Box<dyn StdError + Send + Sync>> {
        let mut cfg = Config::default();

        let endpoints_set: HashSet<_> = self.config.url_mapping
            .name_to_url
            .values()
            .map(|mapped_url| format!("tcp/{}:{}", mapped_url.vpn_ip, mapped_url.vpn_port))
            .collect();

        let mut endpoints: Vec<_> = endpoints_set.into_iter().collect();
        endpoints.sort();

        let endpoints_json = serde_json::to_string(&endpoints)?;
        cfg.insert_json5("connect/endpoints", &endpoints_json)?;
        Ok(cfg)
    }

    pub async fn get_session(&self) -> Result<Session, ZError> {
        let cfg = self.zenoh_config()?;
        zenoh::open(cfg).await
    }

    pub fn get_topic_config_pub(&self, topic_name: &str) -> Option<&TopicConfig> {
        self.config.topics.iter().find(|tc| matches!(tc, TopicConfig::Pub { topic_name: tn, .. } if tn == topic_name))
    }

    pub fn get_topic_config_sub(&self, topic_name: &str) -> Option<&TopicConfig> {
        self.config.topics.iter().find(|tc| matches!(tc, TopicConfig::Sub { topic_name: tn, .. } if tn == topic_name))
    }

    pub fn get_endpoint_config_req(&self, endpoint_name: &str) -> Option<&EndpointConfig> {
        self.config.endpoints.iter().find(|ec| matches!(ec, EndpointConfig::Req { endpoint_name: en, .. } if en == endpoint_name))
    }

    pub fn get_endpoint_config_prv(&self, endpoint_name: &str) -> Option<&EndpointConfig> {
        self.config.endpoints.iter().find(|ec| matches!(ec, EndpointConfig::Prv { endpoint_name: en, .. } if en == endpoint_name))
    }

    pub async fn get_publisher(
        &self,
        session: &Session,
        name: &str,
    ) -> Result<Publisher<'_>, ZError> {
        let config = match self.get_topic_config_pub(name) {
            Some(TopicConfig::Pub { config, .. }) => config,
            _ => return Err(ZenohInterfaceError::PubTopicNotFound(name.to_string()).into()),
        };

        let zenoh_config: ZenohPublisherConfig = decode_config(config)?;

        let publisher = session
            .declare_publisher(name.to_owned()) // Pass &str directly
            .congestion_control(zenoh_config.congestion_control.to_zenoh())
            .priority(zenoh_config.priority.to_zenoh())
            .express(zenoh_config.express)
            .reliability(zenoh_config.reliability.to_zenoh())
            .await?;

        Ok(publisher)
    }

    pub async fn get_subscriber(
        &self,
        session: &Session,
        name: &str,
    ) -> Result<ConfiguredSubscriber, ZError> {
        let config = match self.get_topic_config_sub(name) {
            Some(TopicConfig::Sub { config, .. }) => config,
            _ => return Err(ZenohInterfaceError::SubTopicNotFound(name.to_string()).into()),
        };

        let zenoh_config: ZenohSubscriberConfig = decode_config(config)?;

        match &zenoh_config.handler {
            HandlerChannel::Fifo { capacity } => {
                let subscriber = session
                    .declare_subscriber(name)
                    .with(FifoChannel::new(*capacity as usize))
                    .await?;
                Ok(ConfiguredSubscriber::Fifo(subscriber))
            }
            HandlerChannel::Ring { capacity } => {
                let subscriber = session
                    .declare_subscriber(name)
                    .with(RingChannel::new(*capacity as usize))
                    .await?;
                Ok(ConfiguredSubscriber::Ring(subscriber))
            }
        }
    }

    pub async fn get_subscriber_callback(
        &self,
        session: &Session,
        name: &str,
        handler: Box<dyn Fn(Sample) + Send + Sync + 'static>,
    ) -> Result<Subscriber<()>, ZError> {
        session
            .declare_subscriber(name)
            .callback(handler)
            .await
    }

    pub async fn get_subscriber_callback_mut(
        &self,
        session: &Session,
        name: &str,
        handler: Box<dyn FnMut(Sample) + Send + Sync + 'static>,
    ) -> Result<Subscriber<()>, ZError> {
        session
            .declare_subscriber(name)
            .callback_mut(handler)
            .await
    }

    pub async fn get_requester(
        &self,
        session: &Session,
        name: &str,
    ) -> Result<Querier<'_>, ZError> {
        let config = match self.get_endpoint_config_req(name) {
            Some(EndpointConfig::Req { config, .. }) => config,
            _ => return Err(ZenohInterfaceError::ReqEndpointNotFound(name.to_string()).into()),
        };

        let zenoh_config: ZenohRequesterConfig = decode_config(config)?;

        let querier = session
            .declare_querier(name.to_owned()) // Pass &str directly
            .congestion_control(zenoh_config.congestion_control.to_zenoh())
            .priority(zenoh_config.priority.to_zenoh())
            .express(zenoh_config.express)
            .await?;

        Ok(querier)
    }

    pub async fn get_provider(
        &self,
        session: &Session,
        name: &str,
    ) -> Result<ConfiguredProvider, ZError> {
        let config = match self.get_endpoint_config_prv(name) {
            Some(EndpointConfig::Prv { config, .. }) => config,
            _ => return Err(ZenohInterfaceError::PrvEndpointNotFound(name.to_string()).into()),
        };

        let zenoh_config: ZenohProviderConfig = decode_config(config)?;

        match &zenoh_config.handler {
            HandlerChannel::Fifo { capacity } => {
                let provider = session
                    .declare_queryable(name)
                    .with(FifoChannel::new(*capacity as usize))
                    .await?;
                Ok(ConfiguredProvider::Fifo(provider))
            }
            HandlerChannel::Ring { capacity } => {
                let provider = session
                    .declare_queryable(name)
                    .with(RingChannel::new(*capacity as usize))
                    .await?;
                Ok(ConfiguredProvider::Ring(provider))
            }
        }
    }

    pub async fn get_provider_callback(
        &self,
        session: &Session,
        name: &str,
        handler: Box<dyn Fn(Query) + Send + Sync + 'static>,
    ) -> Result<Queryable<()>, ZError> {
        session
            .declare_queryable(name)
            .callback(handler)
            .await
    }

    pub async fn get_provider_callback_mut(
        &self,
        session: &Session,
        name: &str,
        handler: Box<dyn FnMut(Query) + Send + Sync + 'static>,
    ) -> Result<Queryable<()>, ZError> {
        session
            .declare_queryable(name)
            .callback_mut(handler)
            .await
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::interfaces::zenoh::model::ZenohPublisherConfig;
    use crate::models::{ApplicationConfig, MountedPeripherals, TopicConfig, URLMapping};
    use serde_json::json;
    use std::collections::{BTreeMap, HashMap};
    use zenoh::qos;

    fn default_app_config() -> ApplicationConfig {
        ApplicationConfig {
            topics: vec![],
            endpoints: vec![],
            services: vec![],
            url_mapping: URLMapping { name_to_url: HashMap::new() },
            peripherals: MountedPeripherals { peripherals: vec![] },
            config: Value::Null,
            entrypoint_name: None,
            deployed_application_id: "id1".into(),
            system_id: "sysid".into(),
            deployed_application_name: "app".into(),
            is_release_version: true,
            public_ip: None,
            vpn_ip: "10.0.0.1".into(),
            port_config: vec![],
            git_url: None,
            git_branch: None,
            application_id: "appid".into(),
            application_name: "myapp".into(),
            storage_url: None,
            storage_endpoint_url: None,
            storage_access_key: None,
            storage_secret_key: None,
        }
    }

    fn pub_topic_config() -> TopicConfig {
        TopicConfig::Pub {
            topic_name: "HELLO_WORLD_MESSAGE".into(),
            topic_key: "my_topic_key".into(),
            message_type: "make87_messages.text.text_plain.PlainText".into(),
            interface_name: "zenoh".into(),
            config: {
                let mut m = BTreeMap::new();
                m.insert("congestion_control".to_string(), json!("DROP"));
                m.insert("priority".to_string(), json!("REAL_TIME"));
                m.insert("express".to_string(), json!(true));
                m.insert("reliability".to_string(), json!("BEST_EFFORT"));
                m
            },
            protocol: "zenoh".into(),
            encoding: Some("proto".into()),
        }
    }

    fn sub_topic_config() -> TopicConfig {
        TopicConfig::Sub {
            topic_name: "HELLO_WORLD_MESSAGE".into(),
            topic_key: "my_topic_key".into(),
            message_type: "make87_messages.text.text_plain.PlainText".into(),
            interface_name: "zenoh".into(),
            config: {
                let mut m = BTreeMap::new();
                // Example config for handler = Fifo, capacity = 12
                m.insert("handler".to_string(), json!({"handler_type":"FIFO", "capacity": 12}));
                m
            },
            protocol: "zenoh".into(),
            encoding: Some("proto".into()),
        }
    }

    fn req_endpoint_config() -> EndpointConfig {
        EndpointConfig::Req {
            endpoint_name: "HELLO_WORLD_MESSAGE".into(),
            endpoint_key: "my_req_key".into(),
            requester_message_type: "ReqType".into(),
            provider_message_type: "PrvType".into(),
            interface_name: "zenoh".into(),
            config: {
                let mut m = BTreeMap::new();
                m.insert("congestion_control".to_string(), json!("DROP"));
                m.insert("priority".to_string(), json!("REAL_TIME"));
                m.insert("express".to_string(), json!(true));
                m
            },
            protocol: "zenoh".into(),
            encoding: Some("proto".into()),
        }
    }

    fn prv_endpoint_config() -> EndpointConfig {
        EndpointConfig::Prv {
            endpoint_name: "HELLO_WORLD_MESSAGE".into(),
            endpoint_key: "my_prv_key".into(),
            requester_message_type: "ReqType".into(),
            provider_message_type: "PrvType".into(),
            interface_name: "zenoh".into(),
            config: {
                let mut m = BTreeMap::new();
                m.insert("handler".to_string(), json!({"handler_type":"RING", "capacity": 7}));
                m
            },
            protocol: "zenoh".into(),
            encoding: Some("proto".into()),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_publisher_found() {
        let mut config = default_app_config();
        config.topics.push(pub_topic_config());

        let iface = ZenohInterface::new(config);
        let result = iface.get_topic_config_pub("HELLO_WORLD_MESSAGE");
        assert!(result.is_some());

        // Also test decoding
        if let Some(TopicConfig::Pub { config, .. }) = result {
            let decoded: ZenohPublisherConfig = decode_config(config).unwrap();
            assert_eq!(decoded.priority.to_zenoh(), qos::Priority::RealTime);
            assert!(decoded.express);
        }

        let session = iface.get_session().await.unwrap();
        let publisher = iface.get_publisher(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(publisher.is_ok());
        
        let subscriber = iface.get_subscriber(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(subscriber.is_err());
        let requester = iface.get_requester(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(requester.is_err());
        let provider = iface.get_provider(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(provider.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_subscriber_found() {
        let mut config = default_app_config();
        config.topics.push(sub_topic_config());

        let iface = ZenohInterface::new(config);
        let result = iface.get_topic_config_sub("HELLO_WORLD_MESSAGE");
        assert!(result.is_some());

        // Also test decoding
        if let Some(TopicConfig::Sub { config, .. }) = result {
            use crate::interfaces::zenoh::model::ZenohSubscriberConfig;
            let decoded: ZenohSubscriberConfig = decode_config(config).unwrap();
            match &decoded.handler {
                HandlerChannel::Fifo { capacity } => assert_eq!(*capacity, 12),
                _ => panic!("Expected FIFO handler"),
            }
        }

        let session = iface.get_session().await.unwrap();
        let subscriber = iface.get_subscriber(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(subscriber.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_requester_found() {
        let mut config = default_app_config();
        config.endpoints.push(req_endpoint_config());

        let iface = ZenohInterface::new(config);
        let result = iface.get_endpoint_config_req("HELLO_WORLD_MESSAGE");
        assert!(result.is_some());

        // Also test decoding
        if let Some(EndpointConfig::Req { config, .. }) = result {
            use crate::interfaces::zenoh::model::ZenohRequesterConfig;
            let decoded: ZenohRequesterConfig = decode_config(config).unwrap();
            assert_eq!(decoded.priority.to_zenoh(), qos::Priority::RealTime);
            assert!(decoded.express);
        }

        let session = iface.get_session().await.unwrap();
        let requester = iface.get_requester(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(requester.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_provider_found() {
        let mut config = default_app_config();
        config.endpoints.push(prv_endpoint_config());

        let iface = ZenohInterface::new(config);
        let result = iface.get_endpoint_config_prv("HELLO_WORLD_MESSAGE");
        assert!(result.is_some());

        // Also test decoding
        if let Some(EndpointConfig::Prv { config, .. }) = result {
            use crate::interfaces::zenoh::model::ZenohProviderConfig;
            let decoded: ZenohProviderConfig = decode_config(config).unwrap();
            match &decoded.handler {
                HandlerChannel::Ring { capacity } => assert_eq!(*capacity, 7),
                _ => panic!("Expected RING handler"),
            }
        }

        let session = iface.get_session().await.unwrap();
        let provider = iface.get_provider(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(provider.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_publisher_not_found() {
        let config = default_app_config();
        let iface = ZenohInterface::new(config);
        let result = iface.get_topic_config_pub("DOES_NOT_EXIST");
        assert!(result.is_none());

        let session = iface.get_session().await.unwrap();
        let publisher = iface.get_publisher(&session, "DOES_NOT_EXIST").await;
        assert!(publisher.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_subscriber_returns_none() {
        let config = default_app_config();
        let iface = ZenohInterface::new(config);
        let result = iface.get_topic_config_sub("HELLO_WORLD_MESSAGE");
        assert!(result.is_none());

        let session = iface.get_session().await.unwrap();
        let subscriber = iface.get_subscriber(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(subscriber.is_err());

    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_requester_and_provider_none() {
        let config = default_app_config();
        let iface = ZenohInterface::new(config);

        let req = iface.get_endpoint_config_req("HELLO_WORLD_MESSAGE");
        assert!(req.is_none());

        let prv = iface.get_endpoint_config_prv("HELLO_WORLD_MESSAGE");
        assert!(prv.is_none());

        let session = iface.get_session().await.unwrap();
        let requester = iface.get_requester(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(requester.is_err());
        let provider = iface.get_provider(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(provider.is_err());
    }
}