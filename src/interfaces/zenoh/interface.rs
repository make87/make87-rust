use crate::config::{load_config_from_default_env, ConfigError};
use crate::interfaces::zenoh::model::{
    HandlerChannel, ZenohQueryableConfig, ZenohPublisherConfig, ZenohQuerierConfig,
    ZenohSubscriberConfig,
};
use crate::models::{ApplicationEnvConfig, ProviderEndpointConfig, PublisherTopicConfig};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use zenoh::handlers::{FifoChannel, FifoChannelHandler, RingChannel, RingChannelHandler};
use zenoh::pubsub::{Publisher, Subscriber};
use zenoh::query::{Querier, Query, Queryable};
use zenoh::sample::Sample;
use zenoh::Error as ZError;
use zenoh::{Config, Session};
use thiserror::Error;

fn decode_config<T: serde::de::DeserializeOwned>(
    map: &BTreeMap<String, Value>,
) -> Result<T, ZenohInterfaceError> {
    Ok(serde_json::from_value(Value::Object(
        map.clone().into_iter().collect(),
    ))?)
}

#[derive(Debug, Error)]
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
    Config(#[from] ConfigError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Zenoh(#[from] ZError),
}

pub enum ConfiguredSubscriber {
    Fifo(Subscriber<FifoChannelHandler<Sample>>),
    Ring(Subscriber<RingChannelHandler<Sample>>),
}

pub enum ConfiguredQueryable {
    Fifo(Queryable<FifoChannelHandler<Query>>),
    Ring(Queryable<RingChannelHandler<Query>>),
}

pub struct ZenohInterface {
    config: ApplicationEnvConfig,
    name: String,
}

impl ZenohInterface {
    pub fn new(config: ApplicationEnvConfig, name: &str) -> Self {
        Self {
            config,
            name: name.to_string(),
        }
    }

    pub fn from_default_env(name: &str) -> Result<Self, ZenohInterfaceError> {
        let config = load_config_from_default_env()?;
        Ok(Self {
            config,
            name: name.to_string(),
        })
    }

    pub fn zenoh_config(&self) -> Result<Config, ZenohInterfaceError> {
        let mut cfg = Config::default();
        if !is_port_in_use(7447) {
            let listen_endpoints = vec!["tcp/0.0.0.0:7447"];
            let listen_json = serde_json::to_string(&listen_endpoints)?;
            cfg.insert_json5("listen/endpoints", &listen_json)?;
        }

        let endpoints_set: HashSet<_> = self
            .config
            .interfaces
            .get(&self.name)
            .map(|iface| iface.subscribers.values().map(|s| &s.access_point))
            .into_iter()
            .flatten()
            .map(|ap| format!("tcp/{}:{}", ap.vpn_ip, ap.vpn_port))
            .collect();

        let mut endpoints: Vec<_> = endpoints_set.into_iter().collect();
        endpoints.sort();

        let endpoints_json = serde_json::to_string(&endpoints)?;
        cfg.insert_json5("connect/endpoints", &endpoints_json)?;
        Ok(cfg)
    }

    pub async fn get_session(&self) -> Result<Session, ZenohInterfaceError> {
        let cfg = self.zenoh_config()?;
        let session = zenoh::open(cfg).await?;
        Ok(session)
    }

    pub fn get_publisher_config(&self, topic_name: &str) -> Option<&PublisherTopicConfig> {
        self.config
            .interfaces
            .get(&self.name)?
            .publishers
            .get(topic_name)
    }

    pub fn get_subscriber_config(
        &self,
        topic_name: &str,
    ) -> Option<&crate::models::BoundSubscriber> {
        self.config
            .interfaces
            .get(&self.name)?
            .subscribers
            .get(topic_name)
    }

    pub fn get_requester_config(
        &self,
        endpoint_name: &str,
    ) -> Option<&crate::models::BoundRequester> {
        self.config
            .interfaces
            .get(&self.name)?
            .requesters
            .get(endpoint_name)
    }

    pub fn get_provider_config(&self, endpoint_name: &str) -> Option<&ProviderEndpointConfig> {
        self.config
            .interfaces
            .get(&self.name)?
            .providers
            .get(endpoint_name)
    }

    pub async fn get_publisher(
        &self,
        session: &Session,
        name: &str,
    ) -> Result<Publisher<'static>, ZenohInterfaceError> {
        let pub_cfg = self
            .get_publisher_config(name)
            .ok_or_else(|| ZenohInterfaceError::PubTopicNotFound(name.to_string()))?;
        let zenoh_config: ZenohPublisherConfig = decode_config(&pub_cfg.config)?;
        let publisher = session
            .declare_publisher(pub_cfg.topic_key.clone())
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
    ) -> Result<ConfiguredSubscriber, ZenohInterfaceError> {
        let sub_cfg = self
            .get_subscriber_config(name)
            .ok_or_else(|| ZenohInterfaceError::SubTopicNotFound(name.to_string()))?;
        let zenoh_config: ZenohSubscriberConfig = decode_config(&sub_cfg.config.config)?;
        match &zenoh_config.handler {
            HandlerChannel::Fifo { capacity } => {
                let subscriber = session
                    .declare_subscriber(sub_cfg.config.topic_key.clone())
                    .with(FifoChannel::new(*capacity as usize))
                    .await?;
                Ok(ConfiguredSubscriber::Fifo(subscriber))
            }
            HandlerChannel::Ring { capacity } => {
                let subscriber = session
                    .declare_subscriber(sub_cfg.config.topic_key.clone())
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
    ) -> Result<Subscriber<()>, ZenohInterfaceError> {
        let sub_cfg = self
            .get_subscriber_config(name)
            .ok_or_else(|| ZenohInterfaceError::SubTopicNotFound(name.to_string()))?;
        let subscriber = session
            .declare_subscriber(sub_cfg.config.topic_key.clone())
            .callback(handler)
            .await?;
        Ok(subscriber)
    }

    pub async fn get_subscriber_callback_mut(
        &self,
        session: &Session,
        name: &str,
        handler: Box<dyn FnMut(Sample) + Send + Sync + 'static>,
    ) -> Result<Subscriber<()>, ZenohInterfaceError> {
        let sub_cfg = self
            .get_subscriber_config(name)
            .ok_or_else(|| ZenohInterfaceError::SubTopicNotFound(name.to_string()))?;
        let subscriber = session
            .declare_subscriber(sub_cfg.config.topic_key.clone())
            .callback_mut(handler)
            .await?;
        Ok(subscriber)
    }

    pub async fn get_querier(
        &self,
        session: &Session,
        name: &str,
    ) -> Result<Querier<'static>, ZenohInterfaceError> {
        let req_cfg = self
            .get_requester_config(name)
            .ok_or_else(|| ZenohInterfaceError::ReqEndpointNotFound(name.to_string()))?;
        let zenoh_config: ZenohQuerierConfig = decode_config(&req_cfg.config.config)?;
        let querier = session
            .declare_querier(req_cfg.config.endpoint_key.clone())
            .congestion_control(zenoh_config.congestion_control.to_zenoh())
            .priority(zenoh_config.priority.to_zenoh())
            .express(zenoh_config.express)
            .await?;
        Ok(querier)
    }

    pub async fn get_queryable(
        &self,
        session: &Session,
        name: &str,
    ) -> Result<ConfiguredQueryable, ZenohInterfaceError> {
        let prv_cfg = self
            .get_provider_config(name)
            .ok_or_else(|| ZenohInterfaceError::PrvEndpointNotFound(name.to_string()))?;
        let zenoh_config: ZenohQueryableConfig = decode_config(&prv_cfg.config)?;
        match &zenoh_config.handler {
            HandlerChannel::Fifo { capacity } => {
                let queryable = session
                    .declare_queryable(prv_cfg.endpoint_key.clone())
                    .with(FifoChannel::new(*capacity as usize))
                    .await?;
                Ok(ConfiguredQueryable::Fifo(queryable))
            }
            HandlerChannel::Ring { capacity } => {
                let queryable = session
                    .declare_queryable(prv_cfg.endpoint_key.clone())
                    .with(RingChannel::new(*capacity as usize))
                    .await?;
                Ok(ConfiguredQueryable::Ring(queryable))
            }
        }
    }


    pub async fn get_queryable_callback(
        &self,
        session: &Session,
        name: &str,
        handler: Box<dyn Fn(Query) + Send + Sync + 'static>,
    ) -> Result<Queryable<()>, ZenohInterfaceError> {
        let prv_cfg = self
            .get_provider_config(name)
            .ok_or_else(|| ZenohInterfaceError::PrvEndpointNotFound(name.to_string()))?;
        let queryable = session
            .declare_queryable(prv_cfg.endpoint_key.clone())
            .callback(handler)
            .await?;
        Ok(queryable)
    }

    pub async fn get_queryable_callback_mut(
        &self,
        session: &Session,
        name: &str,
        handler: Box<dyn FnMut(Query) + Send + Sync + 'static>,
    ) -> Result<Queryable<()>, ZenohInterfaceError> {
        let prv_cfg = self
            .get_provider_config(name)
            .ok_or_else(|| ZenohInterfaceError::PrvEndpointNotFound(name.to_string()))?;
        let queryable = session
            .declare_queryable(prv_cfg.endpoint_key.clone())
            .callback_mut(handler)
            .await?;
        Ok(queryable)
    }
}

fn is_port_in_use(port: u16) -> bool {
    let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
    TcpListener::bind(addr).map_err(
        // print the error
        |e| eprintln!("Failed to bind to port {}: {}", port, e),
    ).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interfaces::zenoh::model::ZenohPublisherConfig;
    use crate::models::{
        ApplicationEnvConfig, InterfaceConfig, ProviderEndpointConfig, PublisherTopicConfig,
        RequesterEndpointConfig, SubscriberTopicConfig,
    };
    use crate::models::{ApplicationInfo, MountedPeripherals};
    use serde_json::json;
    use std::collections::BTreeMap;
    use zenoh::qos;

    fn default_app_config() -> ApplicationEnvConfig {
        ApplicationEnvConfig {
            interfaces: BTreeMap::new(),
            peripherals: MountedPeripherals {
                peripherals: vec![],
            },
            config: serde_json::json!({}),
            storage: None,
            application_info: ApplicationInfo {
                deployed_application_id: String::new(),
                deployed_application_name: String::new(),
                system_id: String::new(),
                application_id: String::new(),
                application_name: String::new(),
                git_url: None,
                git_branch: None,
                is_release_version: false,
            },
        }
    }

    fn make_interface_config() -> InterfaceConfig {
        InterfaceConfig {
            name: "zenoh".to_string(),
            publishers: BTreeMap::new(),
            subscribers: BTreeMap::new(),
            requesters: BTreeMap::new(),
            providers: BTreeMap::new(),
            clients: BTreeMap::new(),
            servers: BTreeMap::new(),
        }
    }

    fn pub_topic_config() -> PublisherTopicConfig {
        PublisherTopicConfig {
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

    fn sub_topic_config() -> SubscriberTopicConfig {
        SubscriberTopicConfig {
            topic_name: "HELLO_WORLD_MESSAGE".into(),
            topic_key: "my_topic_key".into(),
            message_type: "make87_messages.text.text_plain.PlainText".into(),
            interface_name: "zenoh".into(),
            config: {
                let mut m = BTreeMap::new();
                m.insert(
                    "handler".to_string(),
                    json!({"handler_type":"FIFO", "capacity": 12}),
                );
                m
            },
            protocol: "zenoh".into(),
            encoding: Some("proto".into()),
        }
    }

    fn req_endpoint_config() -> RequesterEndpointConfig {
        RequesterEndpointConfig {
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

    fn prv_endpoint_config() -> ProviderEndpointConfig {
        ProviderEndpointConfig {
            endpoint_name: "HELLO_WORLD_MESSAGE".into(),
            endpoint_key: "my_prv_key".into(),
            requester_message_type: "ReqType".into(),
            provider_message_type: "PrvType".into(),
            interface_name: "zenoh".into(),
            config: {
                let mut m = BTreeMap::new();
                m.insert(
                    "handler".to_string(),
                    json!({"handler_type":"RING", "capacity": 7}),
                );
                m
            },
            protocol: "zenoh".into(),
            encoding: Some("proto".into()),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_publisher_found() {
        let mut config = default_app_config();
        let mut iface_config = make_interface_config();
        iface_config
            .publishers
            .insert("HELLO_WORLD_MESSAGE".into(), pub_topic_config());
        config.interfaces.insert("zenoh".into(), iface_config);

        let iface = ZenohInterface::new(config, "zenoh");
        let result = iface.get_publisher_config("HELLO_WORLD_MESSAGE");
        assert!(result.is_some());

        // Also test decoding
        if let Some(pub_cfg) = result {
            let decoded: ZenohPublisherConfig = decode_config(&pub_cfg.config).unwrap();
            assert_eq!(decoded.priority.to_zenoh(), qos::Priority::RealTime);
            assert!(decoded.express);
        }

        let session = iface.get_session().await.unwrap();
        let publisher = iface.get_publisher(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(publisher.is_ok());

        let subscriber = iface.get_subscriber(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(subscriber.is_err());
        let requester = iface.get_querier(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(requester.is_err());
        let queryable = iface.get_queryable(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(queryable.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_subscriber_found() {
        let mut config = default_app_config();
        let mut iface_config = make_interface_config();
        iface_config.subscribers.insert(
            "HELLO_WORLD_MESSAGE".into(),
            crate::models::BoundSubscriber {
                access_point: crate::models::AccessPoint {
                    vpn_ip: "127.0.0.1".into(),
                    vpn_port: 7447,
                    public_ip: None,
                    public_port: None,
                    same_node: false,
                },
                config: sub_topic_config(),
            },
        );
        config.interfaces.insert("zenoh".into(), iface_config);

        let iface = ZenohInterface::new(config, "zenoh");
        let result = iface.get_subscriber_config("HELLO_WORLD_MESSAGE");
        assert!(result.is_some());

        // Also test decoding
        if let Some(sub_cfg) = result {
            use crate::interfaces::zenoh::model::ZenohSubscriberConfig;
            let decoded: ZenohSubscriberConfig = decode_config(&sub_cfg.config.config).unwrap();
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
    async fn test_get_querier_found() {
        let mut config = default_app_config();
        let mut iface_config = make_interface_config();
        iface_config.requesters.insert(
            "HELLO_WORLD_MESSAGE".into(),
            crate::models::BoundRequester {
                access_point: crate::models::AccessPoint {
                    vpn_ip: "127.0.0.1".into(),
                    vpn_port: 7447,
                    public_ip: None,
                    public_port: None,
                    same_node: false,
                },
                config: req_endpoint_config(),
            },
        );
        config.interfaces.insert("zenoh".into(), iface_config);

        let iface = ZenohInterface::new(config, "zenoh");
        let result = iface.get_requester_config("HELLO_WORLD_MESSAGE");
        assert!(result.is_some());

        // Also test decoding
        if let Some(req_cfg) = result {
            use crate::interfaces::zenoh::model::ZenohQuerierConfig;
            let decoded: ZenohQuerierConfig = decode_config(&req_cfg.config.config).unwrap();
            assert_eq!(decoded.priority.to_zenoh(), qos::Priority::RealTime);
            assert!(decoded.express);
        }

        let session = iface.get_session().await.unwrap();
        let requester = iface.get_querier(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(requester.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_queryable_found() {
        let mut config = default_app_config();
        let mut iface_config = make_interface_config();
        iface_config
            .providers
            .insert("HELLO_WORLD_MESSAGE".into(), prv_endpoint_config());
        config.interfaces.insert("zenoh".into(), iface_config);

        let iface = ZenohInterface::new(config, "zenoh");
        let result = iface.get_provider_config("HELLO_WORLD_MESSAGE");
        assert!(result.is_some());

        // Also test decoding
        if let Some(prv_cfg) = result {
            use crate::interfaces::zenoh::model::ZenohQueryableConfig;
            let decoded: ZenohQueryableConfig = decode_config(&prv_cfg.config).unwrap();
            match &decoded.handler {
                HandlerChannel::Ring { capacity } => assert_eq!(*capacity, 7),
                _ => panic!("Expected RING handler"),
            }
        }

        let session = iface.get_session().await.unwrap();
        let queryable = iface.get_queryable(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(queryable.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_publisher_not_found() {
        let config = default_app_config();
        let iface = ZenohInterface::new(config, "zenoh");
        let result = iface.get_publisher_config("DOES_NOT_EXIST");
        assert!(result.is_none());

        let session = iface.get_session().await.unwrap();
        let publisher = iface.get_publisher(&session, "DOES_NOT_EXIST").await;
        assert!(publisher.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_subscriber_returns_none() {
        let config = default_app_config();
        let iface = ZenohInterface::new(config, "zenoh");
        let result = iface.get_subscriber_config("HELLO_WORLD_MESSAGE");
        assert!(result.is_none());

        let session = iface.get_session().await.unwrap();
        let subscriber = iface.get_subscriber(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(subscriber.is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_querier_and_queryable_none() {
        let config = default_app_config();
        let iface = ZenohInterface::new(config, "zenoh");

        let req = iface.get_requester_config("HELLO_WORLD_MESSAGE");
        assert!(req.is_none());

        let prv = iface.get_provider_config("HELLO_WORLD_MESSAGE");
        assert!(prv.is_none());

        let session = iface.get_session().await.unwrap();
        let requester = iface.get_querier(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(requester.is_err());
        let queryable = iface.get_queryable(&session, "HELLO_WORLD_MESSAGE").await;
        assert!(queryable.is_err());
    }
}
