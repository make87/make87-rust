use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RerunGRpcServerConfig {
    pub max_bytes: u64,
}
