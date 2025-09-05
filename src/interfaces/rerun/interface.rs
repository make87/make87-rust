use crate::config::{load_config_from_default_env, ConfigError};
use crate::interfaces::rerun::{RerunGRpcClientConfig, RerunGRpcServerConfig};
use crate::models::{ApplicationEnvConfig, BoundClient, ServerServiceConfig};
use rerun::log::ChunkBatcherConfig;
use rerun::{MemoryLimit, RecordingStream, RecordingStreamBuilder, RecordingStreamError};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::Duration;
use uuid::Uuid;

fn decode_config<T: serde::de::DeserializeOwned>(
    map: &BTreeMap<String, Value>,
) -> Result<T, RerunGRpcInterfaceError> {
    Ok(serde_json::from_value(Value::Object(
        map.clone().into_iter().collect(),
    ))?)
}

fn base_recording_builder(system_id: &str) -> RecordingStreamBuilder {
    RecordingStreamBuilder::new(system_id)
        .recording_id(deterministic_uuid_v4_from_string(system_id))
}

#[derive(Debug, thiserror::Error)]
pub enum RerunGRpcInterfaceError {
    #[error("No client service config found with name: {0}")]
    ClientServiceNotFound(String),
    #[error("No server service config found with name: {0}")]
    ServerServiceNotFound(String),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Rerun(#[from] RecordingStreamError),
}

pub struct RerunGRpcInterface {
    config: ApplicationEnvConfig,
    name: String,
}

impl RerunGRpcInterface {
    pub fn new(config: ApplicationEnvConfig, name: &str) -> Self {
        Self {
            config,
            name: name.to_string(),
        }
    }

    pub fn from_default_env(name: &str) -> Result<Self, RerunGRpcInterfaceError> {
        let config = load_config_from_default_env()?;
        Ok(Self {
            config,
            name: name.to_string(),
        })
    }

    pub fn get_client_service_config(&self, topic_name: &str) -> Option<&BoundClient> {
        self.config
            .interfaces
            .get(&self.name)?
            .clients
            .get(topic_name)
    }

    pub fn get_server_service_config(&self, topic_name: &str) -> Option<&ServerServiceConfig> {
        self.config
            .interfaces
            .get(&self.name)?
            .servers
            .get(topic_name)
    }

    pub fn get_client_recording_stream(
        &self,
        name: &str,
    ) -> Result<RecordingStream, RerunGRpcInterfaceError> {
        let client_cfg = self
            .get_client_service_config(name)
            .ok_or_else(|| RerunGRpcInterfaceError::ClientServiceNotFound(name.to_string()))?;

        let rerun_config: RerunGRpcClientConfig = decode_config(&client_cfg.config.config)?;

        // Configure the chunk batcher
        let mut batcher_config = ChunkBatcherConfig::from_env().unwrap_or_default();
        batcher_config.flush_tick =
            Duration::from_secs_f32(rerun_config.batcher_config.flush_tick);
        batcher_config.flush_num_bytes = rerun_config.batcher_config.flush_num_bytes;
        batcher_config.flush_num_rows = rerun_config.batcher_config.flush_num_rows;

        let rec = base_recording_builder(self.config.application_info.system_id.as_str())
            .batcher_config(batcher_config)
            .connect_grpc_opts(
                format!(
                    "rerun+http://{}:{}/proxy",
                    client_cfg.access_point.vpn_ip, client_cfg.access_point.vpn_port
                ),
                rerun_config
                    .flush_timeout
                    .map(|seconds| Duration::from_secs_f32(seconds)),
            )?;

        Ok(rec)
    }

    pub fn get_server_recording_stream(
        &self,
        name: &str,
    ) -> Result<RecordingStream, RerunGRpcInterfaceError> {
        let server_cfg = self
            .get_server_service_config(name)
            .ok_or_else(|| RerunGRpcInterfaceError::ServerServiceNotFound(name.to_string()))?;

        let rerun_config: RerunGRpcServerConfig = decode_config(&server_cfg.config)?;

        let memory_limit = match rerun_config.max_bytes {
            Some(bytes) => MemoryLimit::from_bytes(bytes),
            None => MemoryLimit::from_fraction_of_total(1.0), // No limit
        };

        let rec = base_recording_builder(self.config.application_info.system_id.as_str())
            .serve_grpc_opts("0.0.0.0", 9876, memory_limit)?;
        Ok(rec)
    }
}

fn deterministic_uuid_v4_from_string(s: &str) -> Uuid {
    let hash = Sha256::digest(s.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    bytes[6] = (bytes[6] & 0x0F) | 0x40; // Version 4
    bytes[8] = (bytes[8] & 0x3F) | 0x80; // Variant RFC 4122
    Uuid::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AccessPoint, ApplicationInfo, ClientServiceConfig, InterfaceConfig, MountedPeripherals,
    };
    use serde_json::Value;
    use std::collections::BTreeMap;

    fn create_test_config() -> ApplicationEnvConfig {
        let mut interfaces = BTreeMap::new();

        let mut clients = BTreeMap::new();
        clients.insert(
            "test_client".to_string(),
            BoundClient {
                access_point: AccessPoint {
                    vpn_ip: "127.0.0.1".to_string(),
                    vpn_port: 8080,
                    public_ip: None,
                    public_port: None,
                    same_node: false,
                },
                config: ClientServiceConfig {
                    name: "test_service".to_string(),
                    spec: "test_spec".to_string(),
                    key: "test_key".to_string(),
                    interface_name: "rerun".to_string(),
                    config: BTreeMap::new(),
                    protocol: "grpc".to_string(),
                },
            },
        );

        let mut servers = BTreeMap::new();
        let mut server_config = BTreeMap::new();
        server_config.insert(
            "max_bytes".to_string(),
            Value::Number(serde_json::Number::from(1073741824u64)),
        ); // 1GB

        servers.insert(
            "test_server".to_string(),
            ServerServiceConfig {
                name: "test_server_service".to_string(),
                key: "test_server_key".to_string(),
                spec: "test_server_spec".to_string(),
                interface_name: "rerun".to_string(),
                config: server_config,
                protocol: "grpc".to_string(),
            },
        );

        interfaces.insert(
            "test_interface".to_string(),
            InterfaceConfig {
                name: "test_interface".to_string(),
                publishers: BTreeMap::new(),
                subscribers: BTreeMap::new(),
                requesters: BTreeMap::new(),
                providers: BTreeMap::new(),
                clients,
                servers,
            },
        );

        ApplicationEnvConfig {
            interfaces,
            peripherals: MountedPeripherals {
                peripherals: Vec::new(),
            },
            config: Value::Null,
            storage: None,
            application_info: ApplicationInfo {
                deployed_application_id: "test_deployed_app_id".to_string(),
                deployed_application_name: "test_deployed_app".to_string(),
                system_id: "test_system_id".to_string(),
                application_id: "test_app_id".to_string(),
                application_name: "test_app".to_string(),
                git_url: Some("https://github.com/test/repo".to_string()),
                git_branch: Some("main".to_string()),
                is_release_version: false,
            },
        }
    }

    #[test]
    fn test_new() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config.clone(), "test_interface");

        assert_eq!(interface.name, "test_interface");
        assert_eq!(
            interface.config.application_info.system_id,
            "test_system_id"
        );
    }

    #[test]
    fn test_get_client_config_success() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config, "test_interface");

        let client_config = interface.get_client_service_config("test_client");
        assert!(client_config.is_some());

        let client = client_config.unwrap();
        assert_eq!(client.access_point.vpn_ip, "127.0.0.1");
        assert_eq!(client.access_point.vpn_port, 8080);
        assert_eq!(client.config.name, "test_service");
    }

    #[test]
    fn test_get_client_config_interface_not_found() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config, "nonexistent_interface");

        let client_config = interface.get_client_service_config("test_client");
        assert!(client_config.is_none());
    }

    #[test]
    fn test_get_client_config_client_not_found() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config, "test_interface");

        let client_config = interface.get_client_service_config("nonexistent_client");
        assert!(client_config.is_none());
    }

    #[test]
    fn test_get_client_recording_stream_client_not_found() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config, "test_interface");

        let result = interface.get_client_recording_stream("nonexistent_client");
        assert!(result.is_err());

        match result.unwrap_err() {
            RerunGRpcInterfaceError::ClientServiceNotFound(name) => {
                assert_eq!(name, "nonexistent_client");
            }
            _ => panic!("Expected ClientConfigNotFound error"),
        }
    }

    #[test]
    fn test_get_client_recording_stream_interface_not_found() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config, "nonexistent_interface");

        let result = interface.get_client_recording_stream("test_client");
        assert!(result.is_err());

        match result.unwrap_err() {
            RerunGRpcInterfaceError::ClientServiceNotFound(name) => {
                assert_eq!(name, "test_client");
            }
            _ => panic!("Expected ClientConfigNotFound error"),
        }
    }

    #[test]
    fn test_error_display() {
        let client_error = RerunGRpcInterfaceError::ClientServiceNotFound("test".to_string());
        assert_eq!(
            format!("{}", client_error),
            "No client service config found with name: test"
        );

        let server_error = RerunGRpcInterfaceError::ServerServiceNotFound("test".to_string());
        assert_eq!(
            format!("{}", server_error),
            "No server service config found with name: test"
        );
    }

    #[test]
    fn test_error_from_recording_stream_error() {
        // Test the From trait implementation for RecordingStreamError
        // We'll create a simple error and verify it gets wrapped correctly

        let err: RerunGRpcInterfaceError = serde_json::from_str::<serde_json::Value>("not json")
            .unwrap_err()
            .into();
        match err {
            RerunGRpcInterfaceError::SerdeJson(_) => {}
            _ => panic!("Expected SerdeJson variant"),
        }
    }

    #[test]
    fn test_create_empty_config() {
        let config = ApplicationEnvConfig {
            interfaces: BTreeMap::new(),
            peripherals: MountedPeripherals {
                peripherals: Vec::new(),
            },
            config: Value::Null,
            storage: None,
            application_info: ApplicationInfo {
                deployed_application_id: "empty_test".to_string(),
                deployed_application_name: "empty_test_app".to_string(),
                system_id: "empty_system".to_string(),
                application_id: "empty_app".to_string(),
                application_name: "empty_app_name".to_string(),
                git_url: None,
                git_branch: None,
                is_release_version: true,
            },
        };

        let interface = RerunGRpcInterface::new(config, "empty_interface");
        assert_eq!(interface.name, "empty_interface");

        // Should return None for any client config request
        assert!(interface.get_client_service_config("any_client").is_none());
    }

    #[test]
    fn test_multiple_clients_in_interface() {
        let mut config = create_test_config();

        // Add another client to the same interface
        if let Some(interface_config) = config.interfaces.get_mut("test_interface") {
            interface_config.clients.insert(
                "second_client".to_string(),
                BoundClient {
                    access_point: AccessPoint {
                        vpn_ip: "192.168.1.100".to_string(),
                        vpn_port: 9090,
                        public_ip: Some("203.0.113.1".to_string()),
                        public_port: Some(443),
                        same_node: true,
                    },
                    config: ClientServiceConfig {
                        name: "second_service".to_string(),
                        spec: "second_spec".to_string(),
                        key: "second_key".to_string(),
                        interface_name: "rerun".to_string(),
                        config: BTreeMap::new(),
                        protocol: "http".to_string(),
                    },
                },
            );
        }

        let interface = RerunGRpcInterface::new(config, "test_interface");

        // Both clients should be accessible
        let first_client = interface.get_client_service_config("test_client");
        assert!(first_client.is_some());
        assert_eq!(first_client.unwrap().access_point.vpn_ip, "127.0.0.1");

        let second_client = interface.get_client_service_config("second_client");
        assert!(second_client.is_some());
        assert_eq!(second_client.unwrap().access_point.vpn_ip, "192.168.1.100");
        assert_eq!(
            second_client.unwrap().access_point.public_ip,
            Some("203.0.113.1".to_string())
        );
        assert_eq!(second_client.unwrap().access_point.same_node, true);
    }

    // Server service tests
    #[test]
    fn test_get_server_config_success() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config, "test_interface");

        let server_config = interface.get_server_service_config("test_server");
        assert!(server_config.is_some());

        let server = server_config.unwrap();
        assert_eq!(server.name, "test_server_service");
        assert_eq!(server.key, "test_server_key");
        assert_eq!(server.spec, "test_server_spec");
        assert_eq!(server.protocol, "grpc");
        assert!(server.config.contains_key("max_bytes"));
    }

    #[test]
    fn test_get_server_config_interface_not_found() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config, "nonexistent_interface");

        let server_config = interface.get_server_service_config("test_server");
        assert!(server_config.is_none());
    }

    #[test]
    fn test_get_server_config_server_not_found() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config, "test_interface");

        let server_config = interface.get_server_service_config("nonexistent_server");
        assert!(server_config.is_none());
    }

    #[test]
    fn test_get_server_recording_stream_server_not_found() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config, "test_interface");

        let result = interface.get_server_recording_stream("nonexistent_server");
        assert!(result.is_err());

        match result.unwrap_err() {
            RerunGRpcInterfaceError::ServerServiceNotFound(name) => {
                assert_eq!(name, "nonexistent_server");
            }
            _ => panic!("Expected ServerServiceNotFound error"),
        }
    }

    #[test]
    fn test_get_server_recording_stream_interface_not_found() {
        let config = create_test_config();
        let interface = RerunGRpcInterface::new(config, "nonexistent_interface");

        let result = interface.get_server_recording_stream("test_server");
        assert!(result.is_err());

        match result.unwrap_err() {
            RerunGRpcInterfaceError::ServerServiceNotFound(name) => {
                assert_eq!(name, "test_server");
            }
            _ => panic!("Expected ServerServiceNotFound error"),
        }
    }

    #[test]
    fn test_get_server_recording_stream_invalid_config() {
        let mut config = create_test_config();

        // Create a server with invalid configuration (wrong type for max_bytes)
        if let Some(interface_config) = config.interfaces.get_mut("test_interface") {
            let mut invalid_config = BTreeMap::new();
            invalid_config.insert(
                "max_bytes".to_string(),
                Value::String("not_a_number".to_string()), // Wrong type
            );

            interface_config.servers.insert(
                "invalid_server".to_string(),
                ServerServiceConfig {
                    name: "invalid_server_service".to_string(),
                    key: "invalid_server_key".to_string(),
                    spec: "invalid_server_spec".to_string(),
                    interface_name: "rerun".to_string(),
                    config: invalid_config,
                    protocol: "grpc".to_string(),
                },
            );
        }

        let interface = RerunGRpcInterface::new(config, "test_interface");
        let result = interface.get_server_recording_stream("invalid_server");

        // Should fail due to invalid type in config
        assert!(result.is_err());
        match result.unwrap_err() {
            RerunGRpcInterfaceError::SerdeJson(_) => {
                // Expected - JSON deserialization should fail
            }
            _ => panic!("Expected SerdeJson error for invalid config"),
        }
    }

    #[test]
    fn test_multiple_servers_in_interface() {
        let mut config = create_test_config();

        // Add another server to the same interface
        if let Some(interface_config) = config.interfaces.get_mut("test_interface") {
            let mut second_server_config = BTreeMap::new();
            second_server_config.insert(
                "max_bytes".to_string(),
                Value::Number(serde_json::Number::from(2147483648u64)),
            ); // 2GB

            interface_config.servers.insert(
                "second_server".to_string(),
                ServerServiceConfig {
                    name: "second_server_service".to_string(),
                    key: "second_server_key".to_string(),
                    spec: "second_server_spec".to_string(),
                    interface_name: "rerun".to_string(),
                    config: second_server_config,
                    protocol: "http".to_string(),
                },
            );
        }

        let interface = RerunGRpcInterface::new(config, "test_interface");

        // Both servers should be accessible
        let first_server = interface.get_server_service_config("test_server");
        assert!(first_server.is_some());
        assert_eq!(first_server.unwrap().name, "test_server_service");
        assert_eq!(first_server.unwrap().protocol, "grpc");

        let second_server = interface.get_server_service_config("second_server");
        assert!(second_server.is_some());
        assert_eq!(second_server.unwrap().name, "second_server_service");
        assert_eq!(second_server.unwrap().protocol, "http");

        // Check that the max_bytes config is different
        let first_max_bytes = first_server.unwrap().config.get("max_bytes").unwrap();
        let second_max_bytes = second_server.unwrap().config.get("max_bytes").unwrap();
        assert_ne!(first_max_bytes, second_max_bytes);
    }

    #[test]
    fn test_deterministic_uuid_generation() {
        let system_id = "test_system_id";
        let uuid1 = deterministic_uuid_v4_from_string(system_id);
        let uuid2 = deterministic_uuid_v4_from_string(system_id);

        // Same input should produce same UUID
        assert_eq!(uuid1, uuid2);

        // Different inputs should produce different UUIDs
        let uuid3 = deterministic_uuid_v4_from_string("different_system_id");
        assert_ne!(uuid1, uuid3);

        // Check that it's a valid v4 UUID
        assert_eq!(uuid1.get_version_num(), 4);
    }

    #[test]
    fn test_decode_config_success() {
        let mut config_map = BTreeMap::new();
        config_map.insert(
            "max_bytes".to_string(),
            Value::Number(serde_json::Number::from(1073741824u64)),
        );

        let result: Result<
            crate::interfaces::rerun::RerunGRpcServerConfig,
            RerunGRpcInterfaceError,
        > = decode_config(&config_map);
        assert!(result.is_ok());

        let decoded = result.unwrap();
        assert_eq!(decoded.max_bytes, Some(1073741824u64));
    }

    #[test]
    fn test_decode_config_failure() {
        let mut config_map = BTreeMap::new();
        // Use a field with wrong type to force deserialization error
        config_map.insert(
            "max_bytes".to_string(),
            Value::String("not_a_number".to_string()),
        );

        let result: Result<
            crate::interfaces::rerun::RerunGRpcServerConfig,
            RerunGRpcInterfaceError,
        > = decode_config(&config_map);
        assert!(result.is_err());

        match result.unwrap_err() {
            RerunGRpcInterfaceError::SerdeJson(_) => {
                // Expected - should be a JSON deserialization error
            }
            _ => panic!("Expected SerdeJson error for invalid config decode"),
        }
    }

    #[test]
    fn test_empty_config_server_access() {
        let config = ApplicationEnvConfig {
            interfaces: BTreeMap::new(),
            peripherals: MountedPeripherals {
                peripherals: Vec::new(),
            },
            config: Value::Null,
            storage: None,
            application_info: ApplicationInfo {
                deployed_application_id: "empty_test".to_string(),
                deployed_application_name: "empty_test_app".to_string(),
                system_id: "empty_system".to_string(),
                application_id: "empty_app".to_string(),
                application_name: "empty_app_name".to_string(),
                git_url: None,
                git_branch: None,
                is_release_version: true,
            },
        };

        let interface = RerunGRpcInterface::new(config, "empty_interface");

        // Should return None for any server config request
        assert!(interface.get_server_service_config("any_server").is_none());
    }
}
