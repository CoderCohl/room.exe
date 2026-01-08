use crate::memory::{EntryType, MemoryEntry};
use crate::room::Room;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    pub kv: HashMap<String, String>,
    pub counters: HashMap<String, i64>,
    pub version: String,
}

impl Default for EntityState {
    fn default() -> Self {
        Self { kv: HashMap::new(), counters: HashMap::new(), version: "2.1.0".to_string() }
    }
}

pub struct Entity;

impl Entity {
    pub fn handle_input(room: &mut Room, raw: &str, now: i64) -> Option<String> {
        // Record input
        room.memory.append(MemoryEntry {
            timestamp: now,
            kind: EntryType::INPUT,
            content: raw.to_string(),
            metadata: serde_json::json!({}),
        });

        // Minimal, deterministic protocol: supports a few stateful operations.
        // Everything else is acknowledged or ignored depending on content.
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return None;
        }

        // Hard refusal pattern
        if trimmed.len() > room.config.max_input_size as usize {
            room.memory.append(MemoryEntry {
                timestamp: now,
                kind: EntryType::ERROR,
                content: "ERROR: INPUT_SIZE_EXCEEDED".to_string(),
                metadata: serde_json::json!({"max": room.config.max_input_size}),
            });
            return Some("ERROR: INPUT_SIZE_EXCEEDED".to_string());
        }

        // Commands (entity-level, not process parsing)
        // remember key: value
        if let Some(rest) = trimmed.strip_prefix("remember ") {
            if let Some((k, v)) = rest.split_once(':') {
                let key = k.trim().to_string();
                let val = v.trim().to_string();
                room.entity_state.kv.insert(key, val);
                return Some("ENTITY: Stored.".to_string());
            }
        }

        // recall key
        if let Some(rest) = trimmed.strip_prefix("recall ") {
            let key = rest.trim();
            if let Some(v) = room.entity_state.kv.get(key) {
                return Some(format!("ENTITY: {}", v));
            }
            return Some("ENTITY: No such entry.".to_string());
        }

        // counter operations
        if let Some(rest) = trimmed.strip_prefix("initialize counter ") {
            let name = rest.trim().to_string();
            room.entity_state.counters.insert(name, 0);
            return Some("ENTITY: Counter initialized.".to_string());
        }

        if let Some(rest) = trimmed.strip_prefix("increment counter ") {
            let name = rest.trim().to_string();
            let c = room.entity_state.counters.entry(name.clone()).or_insert(0);
            *c += 1;
            return Some(format!("ENTITY: Counter: {}", *c));
        }

        if let Some(rest) = trimmed.strip_prefix("reset counter ") {
            let name = rest.trim().to_string();
            room.entity_state.counters.insert(name, 0);
            return Some("ENTITY: Counter reset to 0.".to_string());
        }

        if trimmed == "system status" || trimmed == "status" || trimmed == "system check" {
            return Some(format!(
                "ENTITY: Operational. Memory usage {}%. State: {:?}.",
                room.memory_utilization_percent(),
                room.state
            ));
        }

        // Silence heuristic: ignore obvious noise
        if trimmed.len() < 6 && trimmed.chars().all(|c| c.is_ascii_punctuation() || c.is_ascii_digit()) {
            return None;
        }

        Some("ENTITY: Acknowledged.".to_string())
    }
}
