use crate::{ws::ws_manager::Ping, BaseUrl, Message};
use anyhow::{anyhow, Context, Result};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{debug, trace};
use serde::Serialize;
use std::{sync::Arc, time::Duration};
use tokio::{
    net::TcpStream,
    spawn,
    sync::{mpsc, Mutex},
    task::JoinHandle,
    time::{interval, interval_at, Instant},
};
use tokio_tungstenite::{connect_async, tungstenite::protocol, MaybeTlsStream, WebSocketStream};

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type Writer = SplitSink<Socket, protocol::Message>;
type Reader = SplitStream<Socket>;

pub async fn connect(base_url: &BaseUrl) -> Result<Socket> {
    let url = format!("ws{}/ws", &BaseUrl::get_url(base_url)[4..]);

    let (socket, _response) = connect_async(url).await.context("Failed to connect")?;

    Ok(socket)
}

pub async fn send<C: Serialize>(writer: &mut Writer, command: C) -> Result<()> {
    let serialized = serde_json::to_string(&command).context("Failed to serialize command")?;

    trace!("--> {:?}", &serialized);

    writer
        .send(protocol::Message::Text(serialized))
        .await
        .context("Failed to send command")?;

    Ok(())
}

// NOTE: Unknown message types are returned as None
fn parse_message(message: protocol::Message) -> Result<Option<Message>> {
    match message {
        protocol::Message::Text(text) => {
            trace!("<-- {:?}", &text);

            let message = serde_json::from_str::<serde_json::Value>(&text)?;

            match serde_json::from_value::<Message>(message) {
                Ok(message) => Ok(Some(message)),
                Err(e) => {
                    debug!("Unhandled message: {}", e);

                    Ok(None)
                }
            }
        }
        _ => Err(anyhow!("Unhandled message type: {:?}", message)),
    }
}

const PING_INTERVAL: Duration = Duration::from_secs(50);
const PONG_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn stream(
    mut reader: Reader,
    writer: Arc<Mutex<Writer>>,
    tx: mpsc::Sender<Message>,
    mut cancel_rx: mpsc::Receiver<()>,
) -> Result<()> {
    let mut ping_interval = interval(PING_INTERVAL);

    let mut pong_interval = interval_at(Instant::now() + PONG_TIMEOUT, PONG_TIMEOUT);

    loop {
        tokio::select! {
            message = reader.next() => match message {
                None => {
                    trace!("Reader stream ended");
                    break Ok(());
                },
                Some(message) => match message {
                    Err(e) => break Err(e.into()),
                    Ok(message) => {
                        let message = parse_message(message)?;

                        if let Some(message) = message {
                            if let Message::Pong = message {
                                trace!("Pong received. Interval reset");

                                pong_interval = interval_at(
                                    Instant::now() + PONG_TIMEOUT,
                                    PONG_TIMEOUT,
                                );
                            }

                            tx.send(message).await.context("Failed to send message")?;
                        }
                    }
                }
            },
            _ = ping_interval.tick() => {
                send(&mut *writer.lock().await, Ping { method: "ping" }).await?;
            },
            // Handle pong timeout
            _ = pong_interval.tick() => {
                return Err(anyhow!("Pong timeout"));
            },
            _ = cancel_rx.recv() => {
                trace!("Received cancel signal");
                break Ok(());
            }
        }
    }
}

pub async fn connect_and_stream(
    base_url: &BaseUrl,
    inbox_tx: mpsc::Sender<Message>,
    mut outbox_rx: mpsc::Receiver<serde_json::Value>,
    cancel_rx: mpsc::Receiver<()>,
) -> Result<()> {
    let socket = connect(base_url).await?;

    let (writer, reader) = socket.split();
    let writer = Arc::new(Mutex::new(writer));

    tokio::select! {
        result = stream(reader, writer.clone(), inbox_tx, cancel_rx) => result,
        result = async {
            while let Some(message) = outbox_rx.recv().await {
                send(&mut *writer.lock().await, message).await?;
            }

            Ok(())
        } =>
            result

    }
}

pub struct Stream {
    pub outbox_tx: mpsc::Sender<serde_json::Value>,
    cancel_tx: mpsc::Sender<()>,
}

impl Drop for Stream {
    fn drop(&mut self) {
        let cancel_tx = self.cancel_tx.clone();

        spawn(async move {
            let _ = cancel_tx.send(()).await;
        });
    }
}

impl Stream {
    pub fn connect(
        base_url: &BaseUrl,
        inbox_tx: mpsc::Sender<Message>,
    ) -> (Self, JoinHandle<Result<()>>) {
        let (outbox_tx, outbox_rx) = mpsc::channel(100);
        let (cancel_tx, cancel_rx) = mpsc::channel(1);

        let handle = spawn({
            let base_url = *base_url;

            async move { connect_and_stream(&base_url, inbox_tx, outbox_rx, cancel_rx).await }
        });

        (
            Self {
                outbox_tx,
                cancel_tx,
            },
            handle,
        )
    }

    pub async fn send(&self, message: serde_json::Value) -> Result<()> {
        self.outbox_tx
            .send(message)
            .await
            .context("Failed to send message")
    }

    pub async fn cancel(&self) {
        let _ = self.cancel_tx.send(()).await;
    }
}
