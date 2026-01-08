use super::{Persistence, RoomSummary};
use crate::room::{Room, RoomState};
use anyhow::Context;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

pub struct SqlitePersistence {
    db_path: PathBuf,
}

impl SqlitePersistence {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self { db_path: path.as_ref().to_path_buf() }
    }

    fn conn(&self) -> anyhow::Result<Connection> {
        Ok(Connection::open(&self.db_path)?)
    }

    fn migrate(conn: &Connection) -> anyhow::Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS rooms (
              id TEXT PRIMARY KEY,
              state TEXT NOT NULL,
              created_at INTEGER NOT NULL,
              last_active INTEGER NOT NULL,
              room_json TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }
}

impl Persistence for SqlitePersistence {
    fn init(&self) -> anyhow::Result<()> {
        let conn = self.conn()?;
        Self::migrate(&conn)?;
        Ok(())
    }

    fn list_rooms(&self) -> anyhow::Result<Vec<RoomSummary>> {
        self.init()?;
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT room_json FROM rooms")?;
        let mut rows = stmt.query([])?;
        let mut out = vec![];
        while let Some(row) = rows.next()? {
            let raw: String = row.get(0)?;
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
        self.init()?;
        let conn = self.conn()?;
        let raw: String = conn.query_row(
            "SELECT room_json FROM rooms WHERE id = ?1",
            params![id],
            |r| r.get(0),
        ).with_context(|| format!("room not found: {}", id))?;
        Ok(serde_json::from_str(&raw)?)
    }

    fn save_room(&self, room: &Room) -> anyhow::Result<()> {
        self.init()?;
        let conn = self.conn()?;
        let raw = serde_json::to_string(room)?;
        conn.execute(
            "INSERT INTO rooms (id, state, created_at, last_active, room_json)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               state=excluded.state,
               last_active=excluded.last_active,
               room_json=excluded.room_json",
            params![room.id, format!("{:?}", room.state), room.created_at, room.last_active, raw],
        )?;
        Ok(())
    }

    fn delete_room(&self, id: &str) -> anyhow::Result<()> {
        self.init()?;
        let conn = self.conn()?;
        conn.execute("DELETE FROM rooms WHERE id = ?1", params![id])?;
        Ok(())
    }
}
