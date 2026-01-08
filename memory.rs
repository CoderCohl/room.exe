use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    pub entries: Vec<MemoryEntry>,
    pub capacity: u64,
    pub usage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub timestamp: i64,
    pub kind: EntryType,
    pub content: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EntryType {
    INPUT,
    OUTPUT,
    OBSERVATION,
    STATE_CHANGE,
    ERROR,
}

impl MemoryStore {
    pub fn new(capacity: u64) -> Self {
        Self { entries: vec![], capacity, usage: 0 }
    }

    pub fn append(&mut self, entry: MemoryEntry) {
        self.usage = self.usage.saturating_add(entry.content.len() as u64);
        self.entries.push(entry);
    }

    pub fn truncate_to_fit(&mut self) {
        while self.usage > self.capacity && !self.entries.is_empty() {
            let removed = self.entries.remove(0);
            self.usage = self.usage.saturating_sub(removed.content.len() as u64);
        }
    }
}
