use crate::room::{Room, RoomState};
use serde::{Deserialize, Serialize};

pub mod filesystem;
pub mod sqlite;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSummary {
    pub id: String,
    pub state: RoomState,
    pub created_at: i64,
    pub last_active: i64,
    pub memory_usage: u64,
    pub memory_capacity: u64,
    pub total_inputs: u64,
    pub total_outputs: u64,
}

pub trait Persistence: Send + Sync {
    fn init(&self) -> anyhow::Result<()>;
    fn list_rooms(&self) -> anyhow::Result<Vec<RoomSummary>>;
    fn load_room(&self, id: &str) -> anyhow::Result<Room>;
    fn save_room(&self, room: &Room) -> anyhow::Result<()>;
    fn delete_room(&self, id: &str) -> anyhow::Result<()>;
}
