use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name="room.exe", version, about="Backrooms Terminal")]
pub struct Cli {
    #[arg(long)]
    pub config: Option<std::path::PathBuf>,
    #[arg(long)]
    pub verbose: bool,
    #[arg(long)]
    pub no_persist: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init,
    Create {
        #[arg(long)]
        memory_limit: Option<String>,
        #[arg(long)]
        timeout: Option<u64>,
        #[arg(long)]
        compression: Option<String>,
        #[arg(long)]
        name: Option<String>,
    },
    Enter {
        room_id: String,
        #[arg(long)]
        output: Option<std::path::PathBuf>,
        #[arg(long)]
        readonly: bool,
    },
    List {
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        format: Option<String>,
    },
    Inspect {
        room_id: String,
        #[arg(long)]
        format: Option<String>,
    },
    Suspend { room_id: String },
    Resume { room_id: String },
    Destroy { room_id: String, #[arg(long)] confirm: bool },
    Export {
        room_id: String,
        #[arg(long)]
        format: Option<String>,
        #[arg(long)]
        output: std::path::PathBuf,
    },
    Stats { room_id: String },
    Compare { id1: String, id2: String },
    Backup { room_id: String, #[arg(long)] output: std::path::PathBuf },
    Restore { path: std::path::PathBuf },
    Batch { #[arg(long)] file: Option<std::path::PathBuf> },
    #[cfg(feature="daemon")]
    Daemon,
    #[cfg(feature="daemon")]
    Connect,
    Version,
}
