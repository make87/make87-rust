use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RerunGRpcServerConfig {
    #[serde(default)]
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ChunkBatcherConfig {
    pub flush_tick_secs: f32,
    pub flush_num_byte: u64,
    pub flush_num_rows: u64,
}

impl Default for ChunkBatcherConfig {
    fn default() -> Self {
        Self {
            flush_tick_secs: 0.008,
            flush_num_byte: 1048576, // 1MiB
            flush_num_rows: 18446744073709551615,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RerunGRpcClientConfig {
    #[serde(default)]
    pub batcher_config: ChunkBatcherConfig,
    #[serde(default = "default_flush_timeout")]
    pub flush_timeout: Option<f32>,
}

fn default_flush_timeout() -> Option<f32> {
    Some(3.0)
}
