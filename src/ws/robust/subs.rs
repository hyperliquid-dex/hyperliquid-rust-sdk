use super::stream::Stream;
use crate::{BaseUrl, Message, Subscription, SubscriptionSendData};
use anyhow::Result;
use log::{debug, error, trace};
use serde::Serialize;
use std::sync::{atomic::AtomicU32, Arc};
use tokio::{
    spawn,
    sync::{mpsc, oneshot, RwLock},
    task::JoinHandle,
};

type Topic = super::super::ws_manager::Subscription;

// NOTE: Leaking subs can be prevented here by implementing a drop that uses a channel
// to notify the subs manager to remove the sub. This requires Subs to have a handle
pub type SubId = u32;

pub struct Sub {
    pub id: SubId,
    pub topic_key: String,
    pub topic: Topic,
    pub tx: mpsc::UnboundedSender<Message>,
}

#[derive(Serialize, Debug)]
pub struct Unsubscribe {
    pub method: String,
    pub subscription: Topic,
}

enum Command {
    Subscribe {
        subscription: Subscription,
        tx: mpsc::UnboundedSender<Message>,
        reply_tx: oneshot::Sender<SubId>,
    },
    Unsubscribe(SubId),
}

#[derive(Clone)]
pub struct State {
    id_counter: Arc<AtomicU32>,
    subs: Arc<RwLock<Vec<Sub>>>,
}

fn get_topic_key_for_subscription(topic: &Topic) -> String {
    match topic {
        Subscription::UserEvents { user: _ } => "userEvents".to_string(),
        Subscription::OrderUpdates { user: _ } => "orderUpdates".to_string(),
        Subscription::UserFills { user: _ } => "userFills".to_string(),
        _ => serde_json::to_string(topic).expect("Failed to convert subscription to string"),
    }
}

async fn run(
    outbox_tx: mpsc::Sender<serde_json::Value>,
    mut inbox_rx: mpsc::Receiver<Message>,
    mut command_rx: mpsc::Receiver<Command>,
) -> Result<()> {
    let state = State {
        subs: Arc::new(RwLock::new(Vec::new())),
        id_counter: Arc::new(AtomicU32::new(0)),
    };

    loop {
        tokio::select! {
            message = inbox_rx.recv() => {
                match message {
                    Some(message) => {
                        let topic = super::super::WsManager::get_identifier(&message)?;
                            debug!("Received message for topic: {}", topic);

                            for sub in
                                state.subs.read().await
                                .iter()
                                .filter(|s| s.topic_key == topic)
                            {
                                trace!("Sending message to sub ID={}", sub.id);

                                if let Err(e) = sub.tx.send(message.clone()) {
                                    error!(
                                        "Failed to send message for topic {} to sub {}: {}",
                                        topic, sub.id, e
                                    );
                                }
                        }
                    }
                    None => {
                        trace!("Inbox receiver closed");
                        break Ok(());
                    }
                }
            },
            command = command_rx.recv() => {
                match command {
                    Some(Command::Subscribe { subscription, tx, reply_tx }) => {
                        trace!("Received subscribe command for topic: {:?}", &subscription);
                        let id = add(&state, outbox_tx.clone(), subscription, tx).await?;

                        if let Err(e) = reply_tx.send(id) {
                            trace!("Failed to send reply for subscribe command: {}", e);
                        }
                    },
                    Some(Command::Unsubscribe(id)) => {
                        remove(&state, outbox_tx.clone(), id).await?;
                    },
                    None => {}
                }
            },
        }
    }
}

async fn add(
    state: &State,
    outbox_tx: mpsc::Sender<serde_json::Value>,
    topic: Topic,
    tx: mpsc::UnboundedSender<Message>,
) -> Result<SubId> {
    let id = state
        .id_counter
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let topic_key = get_topic_key_for_subscription(&topic);

    let sub = Sub {
        id,
        topic: topic.clone(),
        topic_key: topic_key.clone(),
        tx,
    };

    // NOTE: The mutex is held for the remainder of this function
    let mut subs = state.subs.write().await;

    debug!("Adding sub with id: {} ({})", id, topic_key);

    if !subs.iter().any(|s| s.topic_key == topic_key) {
        debug!("First subscription for this topic, sending subscribe command");

        outbox_tx
            .send(
                serde_json::to_value(SubscriptionSendData {
                    method: "subscribe",
                    subscription: &serde_json::to_value(topic).unwrap(),
                })
                .unwrap(),
            )
            .await?;
    }

    subs.push(sub);

    Ok(id)
}

async fn remove(
    state: &State,
    outbox_tx: mpsc::Sender<serde_json::Value>,
    sub_id: SubId,
) -> Result<()> {
    // Locked for the duration of this function
    let mut subs = state.subs.write().await;

    let (topic, topic_key) = subs
        .iter()
        .find(|s| s.id == sub_id)
        .map(|s| (s.topic.clone(), s.topic_key.clone()))
        .unwrap();

    debug!("Removing sub with id: {} ({})", sub_id, topic_key);

    subs.retain(|s| s.id != sub_id);

    // Send unsub if no subs have topic_key of token.topic_key anymore
    if !subs.iter().any(|s| s.topic_key == topic_key) {
        debug!(
            "Last subscriber removed. Sending unsubscribe for topic: {}",
            topic_key
        );

        outbox_tx
            .send(
                serde_json::to_value(Unsubscribe {
                    method: "unsubscribe".to_string(),
                    subscription: topic,
                })
                .unwrap(),
            )
            .await?;
    }

    Ok(())
}

pub struct Subs {
    stream: Stream,
    command_tx: mpsc::Sender<Command>,
}

pub struct Token {
    id: SubId,
    command_tx: mpsc::Sender<Command>,
}

impl Drop for Token {
    fn drop(&mut self) {
        let (id, command_tx) = (self.id, self.command_tx.clone());

        trace!("Dropping Token with id: {}", self.id);

        spawn(async move {
            let _ = command_tx.send(Command::Unsubscribe(id)).await;
        });
    }
}

impl Subs {
    pub fn start(base_url: &BaseUrl) -> (Self, JoinHandle<Result<()>>) {
        let (inbox_tx, inbox_rx) = mpsc::channel(100);
        let (command_tx, command_rx) = mpsc::channel(100);

        let (stream, stream_handle) = Stream::connect(base_url, inbox_tx);

        let run_handle = run(stream.outbox_tx.clone(), inbox_rx, command_rx);

        let handle = spawn(async {
            tokio::select! {
                result = stream_handle => result.unwrap(),
                result = run_handle => result,
            }
        });

        (Self { stream, command_tx }, handle)
    }

    pub async fn add(&self, topic: Topic, tx: mpsc::UnboundedSender<Message>) -> Result<Token> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.command_tx
            .send(Command::Subscribe {
                subscription: topic,
                tx,
                reply_tx,
            })
            .await?;

        let id = reply_rx.await.map_err(|e| anyhow::anyhow!(e))?;

        Ok(Token {
            id,
            command_tx: self.command_tx.clone(),
        })
    }

    pub async fn remove(&self, sub_id: SubId) -> Result<()> {
        self.command_tx.send(Command::Unsubscribe(sub_id)).await?;

        Ok(())
    }

    pub async fn cancel(&self) {
        self.stream.cancel().await
    }
}
