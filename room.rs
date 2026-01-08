use crate::entity::EntityState;
use crate::memory::MemoryStore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    pub memory_limit: u64,
    pub timeout_seconds: u64,
    pub compression: String,
    pub max_input_size: u64,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            memory_limit: 512 * 1024 * 1024,
            timeout_seconds: 30,
            compression: "zstd".to_string(),
            max_input_size: 65536,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RoomState {
    ACTIVE,
    IDLE,
    SUSPENDED,
    CORRUPTED,
    TERMINATED,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomMetadata {
    pub creation_timestamp: i64,
    pub creator_pid: u32,
    pub creator_user: String,
    pub creator_host: String,
    pub total_inputs: u64,
    pub total_outputs: u64,
    pub total_errors: u64,
    pub last_error: Option<String>,
    pub state_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub created_at: i64,
    pub last_active: i64,
    pub state: RoomState,
    pub config: RoomConfig,
    pub memory: MemoryStore,
    pub entity_state: EntityState,
    pub metadata: RoomMetadata,
}

impl Room {
    pub fn memory_utilization_percent(&self) -> u64 {
        if self.memory.capacity == 0 { return 0; }
        ((self.memory.usage as f64 / self.memory.capacity as f64) * 100.0).round() as u64
    }
}
