mod loader;
mod terminal_ui;

use anyhow::Result;
use clap::Parser;
use loader::{loader, Stat};
use std::sync::Arc;
use terminal_ui::start_terminal;
use tokio::{sync::mpsc::UnboundedSender, task::JoinSet};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long, default_value = "512", env = "BATCH_SIZE")]
    batch_size: usize,
    #[clap(
        short,
        long,
        default_value = "postgres://root:root@127.0.0.1:5432/postgres",
        env = "DATABASE_URL"
    )]
    connection_string: String,
    #[clap(short, long, default_value = "public", env = "DATABASE_SCHEMA")]
    schema: String,
    #[clap(short, long, default_value = "1", env = "NUM_LOADERS")]
    num_loaders: usize,
    #[clap(short, long, default_value = "metrics", env = "ORIGIN")]
    origin: String,
    #[clap(long, default_value=None, env = "NUM_CHUNKS")]
    num_chunks: Option<usize>,
}

fn start_loaders(cli: Cli, tx: UnboundedSender<Stat>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let local = tokio::task::LocalSet::new();
    local
        .block_on(&rt, async move {
            let connection_string = Arc::from(cli.connection_string);
            let origin = Arc::from(cli.origin);
            let schema = Arc::from(cli.schema);
            let mut handles = (0..cli.num_loaders)
                .map(|id| {
                    tokio::spawn(loader(
                        id,
                        Arc::clone(&connection_string),
                        Arc::clone(&schema),
                        Arc::clone(&origin),
                        cli.batch_size,
                        cli.num_chunks,
                        tx.clone(),
                    ))
                })
                .collect::<JoinSet<_>>();
            drop(tx);
            while let Some(res) = handles.join_next().await {
                let _ok = res?;
            }
            Ok::<(), Box<dyn std::error::Error + Send + Sync + 'static>>(())
        })
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}

fn main() -> Result<()> {
    //tracing_subscriber::fmt().with_ansi(false).init();
    let cli = Cli::parse();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let handle = std::thread::spawn(move || start_loaders(cli, tx));
    start_terminal(rx).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let _ = handle.join().map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}
