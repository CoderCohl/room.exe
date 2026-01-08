#[cfg(feature="daemon")]
use crate::config::Config;
#[cfg(feature="daemon")]
use crate::cli::{Cli, Commands};
#[cfg(feature="daemon")]
use crate::config::{Backend};
#[cfg(feature="daemon")]
use crate::persistence::{Persistence};
#[cfg(feature="daemon")]
use anyhow::Context;
#[cfg(feature="daemon")]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(feature="daemon")]
use tokio::net::{TcpListener, TcpStream};

#[cfg(feature="daemon")]
fn persistence_from_cfg(cfg: &Config) -> anyhow::Result<Box<dyn Persistence>> {
    match cfg.persistence.backend {
        Backend::FILESYSTEM => Ok(Box::new(crate::persistence::filesystem::FilesystemPersistence::new(&cfg.persistence.path))),
        Backend::SQLITE => Ok(Box::new(crate::persistence::sqlite::SqlitePersistence::new(&cfg.persistence.path))),
        Backend::LEVELDB => anyhow::bail!("LEVELDB backend not implemented"),
        Backend::REDIS => anyhow::bail!("REDIS backend not implemented"),
    }
}

#[cfg(feature="daemon")]
pub fn run(cfg: Config) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let listener = TcpListener::bind(&cfg.daemon.bind).await?;
        eprintln!("STARTING DAEMON MODE");
        eprintln!("BIND_ADDRESS: {}", cfg.daemon.bind);
        let persistence = persistence_from_cfg(&cfg)?;
        persistence.init()?;

        loop {
            let (socket, addr) = listener.accept().await?;
            let cfg = cfg.clone();
            let persistence = persistence_from_cfg(&cfg)?;
            tokio::spawn(async move {
                if let Err(e) = handle_client(socket, persistence).await {
                    eprintln!("[WARN] client {} error: {}", addr, e);
                }
            });
        }
    })
}

#[cfg(feature="daemon")]
async fn handle_client(socket: TcpStream, persistence: Box<dyn Persistence>) -> anyhow::Result<()> {
    let (r, mut w) = socket.into_split();
    let mut reader = BufReader::new(r);
    w.write_all(b"CONNECTED
").await?;
    w.write_all(b"> ").await?;

    let mut line = String::new();
    while reader.read_line(&mut line).await? != 0 {
        let cmd = line.trim();
        line.clear();

        if cmd.eq_ignore_ascii_case("disconnect") {
            w.write_all(b"SESSION CLOSED
").await?;
            break;
        }
        // Minimal: accept "list" only as proof-of-life.
        if cmd.eq_ignore_ascii_case("list") {
            let rooms = persistence.list_rooms()?;
            w.write_all(format!("ROOMS {}
", rooms.len()).as_bytes()).await?;
            w.write_all(b"> ").await?;
            continue;
        }

        w.write_all(b"UNKNOWN
").await?;
        w.write_all(b"> ").await?;
    }
    Ok(())
}

#[cfg(feature="daemon")]
pub fn connect(cfg: Config) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let mut stream = TcpStream::connect(&cfg.daemon.bind).await.context("connect failed")?;
        let (r, mut w) = stream.split();
        let mut reader = BufReader::new(r);

        let mut banner = String::new();
        reader.read_line(&mut banner).await?;
        print!("{}", banner);

        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            w.write_all(input.as_bytes()).await?;
            let mut resp = String::new();
            reader.read_line(&mut resp).await?;
            print!("{}", resp);
        }
    })
}
