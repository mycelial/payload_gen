use std::{
    future::pending,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use section::{
    dummy::DummySectionChannel,
    message::{Chunk, Column, DataFrame, DataType, Message, ValueView},
    section::Section,
};
use tokio::sync::mpsc::{channel, UnboundedSender};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug)]
pub struct XorShift {
    state: u64,
}

pub struct Stat {
    pub size: usize,
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
    origin: Arc<str>,
    inner: Option<Box<dyn DataFrame>>,
}

impl Msg {
    fn new(rng: &mut XorShift, batch_size: usize, origin: Arc<str>) -> Self {
        Self {
            origin,
            inner: Some(Box::new(Df::new(rng, batch_size))),
        }
    }
}

impl Message for Msg {
    fn ack(&mut self) -> section::message::Ack {
        Box::pin(async move {})
    }

    fn next(&mut self) -> section::message::Next<'_> {
        let payload = Ok(self.inner.take().map(Chunk::DataFrame));
        Box::pin(async move { payload })
    }

    fn origin(&self) -> &str {
        &self.origin
    }
}

#[derive(Debug)]
struct Df {
    values: Vec<String>,
}

impl Df {
    fn new(rng: &mut XorShift, batch_size: usize) -> Self {
        static CHARS: &[char] = &[
            'a', 'b', 'c', 'd', 'e', 'f', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        ];
        let values = (0..batch_size).fold(Vec::with_capacity(batch_size), |mut acc, _| {
            let len = 16; //(rng.next().max(1) % 255) as usize;
            let value = (0..len)
                .map(|_| CHARS[(rng.next() as usize) % CHARS.len()])
                .collect::<String>();
            acc.push(value);
            acc
        });
        Self { values }
    }
}

impl DataFrame for Df {
    fn columns(&self) -> Vec<section::message::Column<'_>> {
        vec![Column::new(
            "value",
            DataType::Str,
            Box::new(self.values.iter().map(|v| ValueView::Str(v))),
        )]
    }
}

pub async fn loader(
    _id: usize,
    connection_string: Arc<str>,
    schema: Arc<str>,
    origin: Arc<str>,
    batch_size: usize,
    num_chunks: Option<usize>,
    sender: UnboundedSender<Stat>,
) -> Result<()> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let mut rng = XorShift::new(now.as_nanos() as u64);
    let postgres_dst =
        postgres_connector::destination::Postgres::new(&*connection_string, &*schema, false, None);

    let (tx, rx) = channel(1);
    let rx = ReceiverStream::new(rx);

    let pg_handle =
        tokio::spawn(postgres_dst.start(rx, stub::Stub::new(), DummySectionChannel::new()));

    let mut count = 0;
    loop {
        if let Some(num_chunks) = num_chunks {
            if count >= num_chunks {
                drop(sender);
                break;
            }
        }
        count += 1;
        match tx
            .send(Box::new(Msg::new(
                &mut rng,
                batch_size,
                Arc::clone(&origin),
            )))
            .await
        {
            Ok(_) => sender.send(Stat { size: batch_size })?,
            Err(_) => {
                let res = pg_handle.await;
                return Err(anyhow::anyhow!("failed to sent message: {res:?}"));
            }
        };
    }
    pending::<()>().await;
    Ok(())
}
