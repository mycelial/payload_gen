use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use anyhow::Result;
use clap::Parser;
use section::{
    dummy::DummySectionChannel, futures::{self, Sink, Stream}, message::Message, section::Section, SectionError, SectionFuture, SectionMessage
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::PollSender;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long, default_value = "512")]
    batch_size: usize,
    #[clap(short, long, default_value = "postgres://root:root@127.0.0.1:5432/postgres")]
    connection_string: String,
    #[clap(short, long, default_value = "PUBLIC")]
    schema: String,
}

struct XorShift {
    state: u64,
}

impl XorShift {
    fn new(state: u64) -> Self {
        Self {
            state: state.max(1),
        }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        self.state
    }
}


#[derive(Debug)]
struct Msg {
    inner: Option<Df>
}

impl Msg {
    fn new(rng: &mut XorShift, batch_size: usize) -> Self {
        Self { inner: Some(Df::new(rng, batch_size)) }
    }
}

impl Message for Msg {
    fn ack(&mut self) -> section::message::Ack {
        Box::pin(async move {})
    }

    fn next(&mut self) -> section::message::Next<'_> {
        let payload = Ok(self.inner.take());
        Box::pin(async move {
            payload
        })
    }

    fn origin(&self) -> &str {
        "payload_gen"
    }
}

#[derive(Debug)]
struct Df {}

impl Df {
    fn new(rng: &mut XorShift, batch_size: usize) -> Self {
        Self {}
    }
}

#[derive(Debug)]
struct PayloadGen {}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let mut rng = XorShift::new(now.as_secs());
    tracing::info!("cli: {:?}", cli);

    let postgres_dst = postgres_connector::destination::Postgres::new(
        &cli.connection_string,
        &cli.schema,
        false
    );

    let pg_handle = tokio::spawn(postgres_dst.start(
        stub::Stub::<SectionMessage, SectionError>::new(),
        stub::Stub::new(),
        DummySectionChannel::new()
    ));

    pg_handle.await?.map_err(|e| anyhow::anyhow!(e))?;
    tracing::info!("done");

    Ok(())
}
