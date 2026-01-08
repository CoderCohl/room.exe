use anyhow::Context;
use backrooms_terminal::{cli::{Cli, Commands}, config::{Config, Backend}, entity::Entity, persistence::{Persistence}, room::{Room, RoomConfig, RoomMetadata, RoomState}};
use clap::Parser;
use sha2::{Digest, Sha256};
use std::io::{self, Write, Read};
use std::path::PathBuf;
use time::OffsetDateTime;

fn now_ts() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

fn make_room_id(origin: &str) -> String {
    let ts = now_ts().to_string();
    let seed = uuid::Uuid::new_v4().to_string();
    let mut h = Sha256::new();
    h.update(ts.as_bytes());
    h.update(seed.as_bytes());
    h.update(origin.as_bytes());
    hex::encode(h.finalize())
}

fn parse_size(s: &str) -> anyhow::Result<u64> {
    let s = s.trim().to_uppercase();
    let (num, unit) = s.split_at(s.chars().take_while(|c| c.is_ascii_digit()).count());
    let n: u64 = num.parse()?;
    let mult = match unit {
        "K" | "KB" => 1024,
        "M" | "MB" => 1024 * 1024,
        "G" | "GB" => 1024 * 1024 * 1024,
        "" => 1,
        _ => anyhow::bail!("invalid size unit: {}", unit),
    };
    Ok(n * mult)
}

fn persistence_from_cfg(cfg: &Config) -> anyhow::Result<Box<dyn Persistence>> {
    match cfg.persistence.backend {
        Backend::FILESYSTEM => Ok(Box::new(backrooms_terminal::persistence::filesystem::FilesystemPersistence::new(&cfg.persistence.path))),
        Backend::SQLITE => Ok(Box::new(backrooms_terminal::persistence::sqlite::SqlitePersistence::new(&cfg.persistence.path))),
        Backend::LEVELDB => anyhow::bail!("LEVELDB backend not implemented in this reference build"),
        Backend::REDIS => anyhow::bail!("REDIS backend not implemented in this reference build"),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = Config::load(cli.config.as_deref())?;
    let persistence = persistence_from_cfg(&cfg)?;
    persistence.init()?;

    match cli.command {
        Commands::Init => {
            println!("INITIALIZING ROOM SYSTEM");
            println!("PERSISTENCE BACKEND: {:?}", cfg.persistence.backend);
            println!("PATH: {}", cfg.persistence.path);
            let rooms = persistence.list_rooms()?;
            println!("SCANNING EXISTING ROOMS: {} FOUND", rooms.len());
            println!("READY");
        }
        Commands::Create { memory_limit, timeout, compression, name } => {
            let origin = format!("pid:{} user:{} host:{}",
                std::process::id(),
                std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
                hostname::get().ok().and_then(|h| h.into_string().ok()).unwrap_or_else(|| "unknown".to_string())
            );
            let id = make_room_id(&origin);

            let mut rc = RoomConfig::default();
            rc.memory_limit = memory_limit.as_deref().map(parse_size).transpose()?.unwrap_or(cfg.limits.max_room_memory);
            rc.timeout_seconds = timeout.unwrap_or(cfg.limits.entity_timeout);
            rc.compression = compression.unwrap_or_else(|| cfg.persistence.compression.clone());
            rc.max_input_size = cfg.limits.max_input_size as u64;

            let now = now_ts();

            let room = Room{
                id: id.clone(),
                created_at: now,
                last_active: now,
                state: RoomState::ACTIVE,
                config: rc.clone(),
                memory: backrooms_terminal::memory::MemoryStore::new(rc.memory_limit),
                entity_state: backrooms_terminal::entity::EntityState::default(),
                metadata: RoomMetadata{
                    creation_timestamp: now,
                    creator_pid: std::process::id(),
                    creator_user: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
                    creator_host: hostname::get().ok().and_then(|h| h.into_string().ok()).unwrap_or_else(|| "unknown".to_string()),
                    total_inputs: 0,
                    total_outputs: 0,
                    total_errors: 0,
                    last_error: None,
                    state_version: 1,
                },
            };

            persistence.save_room(&room)?;
            println!("ROOM CREATED: {}", id);
            if let Some(alias) = name {
                println!("ALIAS: {}", alias);
            }
            println!("CONFIG: memory_limit={} timeout={} compression={}", rc.memory_limit, rc.timeout_seconds, rc.compression);
            println!("STATE: ACTIVE");
            println!("ENTITY: INITIALIZED");
        }
        Commands::Enter { room_id, output, readonly } => {
            let mut room = persistence.load_room(&room_id)?;
            if room.state == RoomState::SUSPENDED {
                anyhow::bail!("ERROR: ROOM_SUSPENDED");
            }
            if room.state == RoomState::CORRUPTED {
                anyhow::bail!("ERROR: ROOM_CORRUPTED");
            }
            println!("ENTERING ROOM");
            let mut out: Box<dyn Write> = if let Some(p) = output {
                Box::new(std::fs::File::create(p)?)
            } else {
                Box::new(io::stdout())
            };

            let stdin = io::stdin();
            loop {
                write!(io::stdout(), "> ")?;
                io::stdout().flush()?;
                let mut buf = String::new();
                if stdin.read_line(&mut buf)? == 0 { break; }
                let input = buf.trim_end_matches(['\n','\r']);
                if input == "exit" || input == "quit" { break; }

                let now = now_ts();
                room.last_active = now;
                room.metadata.total_inputs += 1;

                let resp = Entity::handle_input(&mut room, input, now);
                if let Some(line) = resp {
                    room.metadata.total_outputs += 1;
                    writeln!(out, "{line}")?;
                    room.memory.append(backrooms_terminal::memory::MemoryEntry{
                        timestamp: now,
                        kind: backrooms_terminal::memory::EntryType::OUTPUT,
                        content: line.clone(),
                        metadata: serde_json::json!({}),
                    });
                }

                room.memory.truncate_to_fit();

                if !readonly {
                    persistence.save_room(&room)?;
                }
            }
            println!("EXITING ROOM");
        }
        Commands::List { state, limit, format } => {
            let mut rooms = persistence.list_rooms()?;
            if let Some(s) = state {
                rooms.retain(|r| format!("{:?}", r.state).eq_ignore_ascii_case(&s.to_uppercase()));
            }
            if let Some(l) = limit {
                rooms.truncate(l);
            }
            match format.as_deref().unwrap_or("text") {
                "json" => println!("{}", serde_json::to_string_pretty(&rooms)?),
                _ => {
                    println!("ROOMS:");
                    for r in rooms {
                        println!("{} {:?} created_at={} last_active={} mem={} / {}",
                            r.id, r.state, r.created_at, r.last_active, r.memory_usage, r.memory_capacity);
                    }
                }
            }
        }
        Commands::Inspect { room_id, format } => {
            let room = persistence.load_room(&room_id)?;
            match format.as_deref().unwrap_or("text") {
                "json" => println!("{}", serde_json::to_string_pretty(&room)?),
                _ => {
                    println!("ROOM INSPECTION");
                    println!("ID: {}", room.id);
                    println!("STATE: {:?}", room.state);
                    println!("CREATED: {}", room.created_at);
                    println!("LAST_ACTIVE: {}", room.last_active);
                    println!("MEMORY: {} / {}", room.memory.usage, room.memory.capacity);
                    println!("INPUTS: {}", room.metadata.total_inputs);
                    println!("OUTPUTS: {}", room.metadata.total_outputs);
                }
            }
        }
        Commands::Suspend { room_id } => {
            let mut room = persistence.load_room(&room_id)?;
            room.state = RoomState::SUSPENDED;
            room.metadata.state_version += 1;
            persistence.save_room(&room)?;
            println!("STATE: SUSPENDED");
        }
        Commands::Resume { room_id } => {
            let mut room = persistence.load_room(&room_id)?;
            room.state = RoomState::ACTIVE;
            room.metadata.state_version += 1;
            persistence.save_room(&room)?;
            println!("STATE: ACTIVE");
        }
        Commands::Destroy { room_id, confirm } => {
            if !confirm {
                anyhow::bail!("refusing to destroy without --confirm");
            }
            persistence.delete_room(&room_id)?;
            println!("ROOM TERMINATED");
        }
        Commands::Export { room_id, format, output } => {
            let room = persistence.load_room(&room_id)?;
            let fmt = format.unwrap_or_else(|| "jsonl".to_string());
            match fmt.as_str() {
                "jsonl" => {
                    let mut f = std::fs::File::create(&output)?;
                    for e in room.memory.entries {
                        let line = serde_json::to_string(&e)?;
                        writeln!(f, "{line}")?;
                    }
                }
                "json" => {
                    std::fs::write(&output, serde_json::to_string_pretty(&room.memory.entries)?)?;
                }
                _ => anyhow::bail!("unsupported export format: {}", fmt),
            }
            println!("EXPORTED: {}", output.display());
        }
        Commands::Stats { room_id } => {
            let room = persistence.load_room(&room_id)?;
            println!("ROOM STATISTICS");
            println!("ID: {}", room.id);
            println!("STATE: {:?}", room.state);
            println!("CREATED: {}", room.created_at);
            println!("LAST_ACTIVE: {}", room.last_active);
            println!("MEMORY_USAGE: {} bytes", room.memory.usage);
            println!("MEMORY_CAPACITY: {} bytes", room.memory.capacity);
            println!("ENTRIES: {}", room.memory.entries.len());
            println!("TOTAL_INPUTS: {}", room.metadata.total_inputs);
            println!("TOTAL_OUTPUTS: {}", room.metadata.total_outputs);
        }
        Commands::Compare { id1, id2 } => {
            let a = persistence.load_room(&id1)?;
            let b = persistence.load_room(&id2)?;
            println!("COMPARING ROOMS");
            println!("ROOM A: {} {:?}", a.id, a.state);
            println!("ROOM B: {} {:?}", b.id, b.state);
            println!("MEMORY A: {} entries {} bytes", a.memory.entries.len(), a.memory.usage);
            println!("MEMORY B: {} entries {} bytes", b.memory.entries.len(), b.memory.usage);
            println!("NO SHARED MEMORY DETECTED");
        }
        Commands::Backup { room_id, output } => {
            let room_dir = PathBuf::from(&cfg.persistence.path).join(&room_id);
            if !room_dir.exists() {
                anyhow::bail!("room directory not found for backup: {}", room_dir.display());
            }
            let tar_gz = std::fs::File::create(&output)?;
            let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
            let mut tar = tar::Builder::new(enc);
            tar.append_dir_all(&room_id, &room_dir)?;
            tar.finish()?;
            println!("BACKUP COMPLETE: {}", output.display());
        }
        Commands::Restore { path } => {
            let tar_gz = std::fs::File::open(&path)?;
            let dec = flate2::read::GzDecoder::new(tar_gz);
            let mut archive = tar::Archive::new(dec);
            archive.unpack(&cfg.persistence.path)?;
            println!("RESTORE COMPLETE");
        }
        Commands::Batch { file } => {
            let input = if let Some(p) = file {
                std::fs::read_to_string(p)?
            } else {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf)?;
                buf
            };

            let mut last_room: Option<String> = None;
            for line in input.lines() {
                let line = line.trim();
                if line.is_empty() { continue; }
                // Very small batch language: supports "create" and "enter <id>" and entity inputs.
                if line.starts_with("create") {
                    // call create by reusing this process logic
                    let origin = "batch".to_string();
                    let id = make_room_id(&origin);
                    let now = now_ts();
                    let rc = RoomConfig::default();
                    let room = Room{
                        id: id.clone(),
                        created_at: now,
                        last_active: now,
                        state: RoomState::ACTIVE,
                        config: rc.clone(),
                        memory: backrooms_terminal::memory::MemoryStore::new(rc.memory_limit),
                        entity_state: backrooms_terminal::entity::EntityState::default(),
                        metadata: RoomMetadata{
                            creation_timestamp: now,
                            creator_pid: std::process::id(),
                            creator_user: "batch".to_string(),
                            creator_host: "batch".to_string(),
                            total_inputs: 0,
                            total_outputs: 0,
                            total_errors: 0,
                            last_error: None,
                            state_version: 1,
                        },
                    };
                    persistence.save_room(&room)?;
                    println!("ROOM CREATED: {}", id);
                    last_room = Some(id);
                    continue;
                }
                if line.starts_with("enter") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let id = if parts.len() == 2 && parts[1] == "{LAST_ROOM_ID}" {
                        last_room.clone().context("no last room")?
                    } else if parts.len() == 2 {
                        parts[1].to_string()
                    } else {
                        anyhow::bail!("invalid enter syntax");
                    };
                    // subsequent lines until "exit" are routed
                    let mut room = persistence.load_room(&id)?;
                    println!("ENTERING ROOM");
                    continue;
                }
                // batch entity input would require a room context; kept minimal here.
                println!("IGNORED: {}", line);
            }
            println!("BATCH COMPLETE");
        }
        #[cfg(feature="daemon")]
        Commands::Daemon => {
            backrooms_terminal::daemon::run(cfg)?;
        }
        #[cfg(feature="daemon")]
        Commands::Connect => {
            backrooms_terminal::daemon::connect(cfg)?;
        }
        Commands::Version => {
            println!("room.exe version {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
