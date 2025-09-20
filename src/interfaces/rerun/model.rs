use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RerunGRpcServerConfig {
    #[serde(default)]
    pub memory_limit: Option<u64>,
    #[serde(default)]
    pub playback_behavior: PlaybackBehavior,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum PlaybackBehavior {
    /// Start playing back all the old data first,
    /// and only after start sending anything that happened since.
    OldestFirst,
    /// Prioritize the newest arriving messages,
    /// replaying the history later, starting with the newest.
    NewestFirst,
}

impl Default for PlaybackBehavior {
    fn default() -> Self {
        Self::OldestFirst
    }
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
