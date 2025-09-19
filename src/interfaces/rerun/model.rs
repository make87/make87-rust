use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RerunGRpcServerConfig {
    #[serde(default)]
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ChunkBatcherConfig {
    pub flush_tick: f32,
    pub flush_num_bytes: u64,
    pub flush_num_rows: u64,
}

impl Default for ChunkBatcherConfig {
    fn default() -> Self {
        Self {
            flush_tick: 0.2, // 200ms
            flush_num_bytes: 1048576, // 1MiB
            flush_num_rows: 18446744073709551615, // u64::MAX
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RerunGRpcClientConfig {
    #[serde(default)]
    pub batcher_config: ChunkBatcherConfig,
}
