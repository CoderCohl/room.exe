use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub persistence: PersistenceConfig,
    pub limits: LimitsConfig,
    pub logging: LoggingConfig,
    pub entity: EntityConfig,
    pub daemon: DaemonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub backend: Backend,
    pub path: String,
    pub flush_interval: u64,
    pub compression: String,
    pub backup: BackupConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub interval: u64,
    pub retention: u64,
    pub path: String,
    pub compression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    pub max_rooms: usize,
    pub max_room_memory: u64,
    pub max_input_size: usize,
    pub max_output_size: usize,
    pub entity_timeout: u64,
    pub input_queue_depth: usize,
    pub memory_entries_max: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityConfig {
    pub init_timeout: u64,
    pub response_buffer: usize,
    pub memory_compression_threshold: f64,
    pub enable_observations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub enabled: bool,
    pub bind: String,
    pub workers: usize,
    pub max_connections: usize,
    pub connection_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Backend {
    FILESYSTEM,
    SQLITE,
    LEVELDB,
    REDIS,
}

fn home_config_path() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config/room.exe/config.json"))
}

impl Config {
    pub fn load(explicit: Option<&Path>) -> anyhow::Result<Self> {
        let mut candidates: Vec<PathBuf> = vec![PathBuf::from("config/default.json")];

        if let Some(p) = home_config_path() {
            candidates.insert(0, p);
        }
        candidates.insert(0, PathBuf::from("/etc/room.exe/config.json"));

        if let Some(p) = explicit {
            candidates.insert(0, p.to_path_buf());
        }

        for p in candidates {
            if p.exists() {
                let raw = std::fs::read_to_string(&p)?;
                return Ok(serde_json::from_str(&raw)?);
            }
        }

        let raw = include_str!("../config/default.json");
        Ok(serde_json::from_str(raw)?)
    }
}
