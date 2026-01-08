use super::{Persistence, RoomSummary};
use crate::room::{Room, RoomState};
use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FilesystemPersistence {
    root: PathBuf,
}

impl FilesystemPersistence {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf() }
    }

    fn room_dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }
}

impl Persistence for FilesystemPersistence {
    fn init(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.root)?;
        Ok(())
    }

    fn list_rooms(&self) -> anyhow::Result<Vec<RoomSummary>> {
        self.init()?;
        let mut out = vec![];
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() { continue; }
            let id = entry.file_name().to_string_lossy().to_string();
            let meta_path = self.room_dir(&id).join("room.json");
            if !meta_path.exists() { continue; }
            let raw = fs::read_to_string(&meta_path)?;
            let room: Room = serde_json::from_str(&raw)?;
            out.push(RoomSummary{
                id: room.id.clone(),
                state: room.state,
                created_at: room.created_at,
                last_active: room.last_active,
                memory_usage: room.memory.usage,
                memory_capacity: room.memory.capacity,
                total_inputs: room.metadata.total_inputs,
                total_outputs: room.metadata.total_outputs,
            });
        }
        Ok(out)
    }

    fn load_room(&self, id: &str) -> anyhow::Result<Room> {
        let path = self.room_dir(id).join("room.json");
        let raw = fs::read_to_string(&path).with_context(|| format!("missing room state: {}", path.display()))?;
        Ok(serde_json::from_str(&raw)?)
    }

    fn save_room(&self, room: &Room) -> anyhow::Result<()> {
        let dir = self.room_dir(&room.id);
        fs::create_dir_all(&dir)?;
        let path = dir.join("room.json");
        let raw = serde_json::to_string_pretty(room)?;
        fs::write(&path, raw)?;
        Ok(())
    }

    fn delete_room(&self, id: &str) -> anyhow::Result<()> {
        let dir = self.room_dir(id);
        if dir.exists() {
            fs::remove_dir_all(dir)?;
        }
        Ok(())
    }
}
