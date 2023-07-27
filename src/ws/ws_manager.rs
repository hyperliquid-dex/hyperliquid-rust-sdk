use crate::{prelude::*, ws::subscription_response_types::Trades, Error};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use log::error;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    net::TcpStream,
    spawn,
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, protocol::Message},
    MaybeTlsStream, WebSocketStream,
};

struct SubscriptionData {
    sending_channel: mpsc::UnboundedSender<String>,
    subscription_id: u32,
}
pub(crate) struct WsManager {
    writer: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    subscriptions: Arc<Mutex<HashMap<String, Vec<SubscriptionData>>>>,
    subscription_id: u32,
    subscription_identifiers: HashMap<u32, String>,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionType {
    AllMids,
    Trades { coin: String },
}

#[derive(Deserialize)]
#[serde(tag = "channel")]
#[serde(rename_all = "camelCase")]
enum SubscriptionResponseTypes {
    AllMids,
    Trades(Trades),
    SubscriptionResponse,
}

#[derive(Serialize)]
pub(crate) struct WebsocketData<'a> {
    method: &'static str,
    subscription: &'a serde_json::Value,
}

impl WsManager {
    fn get_identifier(response: SubscriptionResponseTypes) -> Result<String> {
        match response {
            SubscriptionResponseTypes::AllMids => Ok("allMids".to_string()),
            SubscriptionResponseTypes::Trades(trades) => {
                if trades.data.is_empty() {
                    Ok(String::default())
                } else {
                    serde_json::to_string(&SubscriptionType::Trades {
                        coin: trades.data[0].coin.clone(),
                    })
                    .map_err(|e| Error::JsonParse(e.to_string()))
                }
            }
            SubscriptionResponseTypes::SubscriptionResponse => Ok(String::default()),
        }
    }

    async fn parse_data(
        data: Option<std::result::Result<Message, tungstenite::Error>>,
        subscriptions: &Arc<Mutex<HashMap<String, Vec<SubscriptionData>>>>,
    ) -> Result<()> {
        let data = data
            .ok_or(Error::ReaderDataNotFound)?
            .map_err(|e| Error::GenericReader(e.to_string()))?
            .into_text()
            .map_err(|e| Error::ReaderTextConversion(e.to_string()))?;

        if !data.starts_with('{') {
            return Ok(());
        }

        let response = serde_json::from_str::<SubscriptionResponseTypes>(&data)
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        let identifier = WsManager::get_identifier(response)?;
        if identifier.is_empty() {
            return Ok(());
        }

        let mut subscriptions = subscriptions.lock().await;
        let mut res = Ok(());
        if let Some(subscription_datas) = subscriptions.get_mut(&identifier) {
            for subscription_data in subscription_datas {
                if let Err(e) = subscription_data
                    .sending_channel
                    .send(data.clone())
                    .map_err(|e| Error::WsSend(e.to_string()))
                {
                    res = Err(e);
                }
            }
        }
        res
    }
    pub(crate) async fn new(url: String) -> Result<WsManager> {
        let (ws_stream, _) = connect_async(url.clone())
            .await
            .map_err(|e| Error::Websocket(e.to_string()))?;

        let (writer, mut reader) = ws_stream.split();

        let subscriptions_map: HashMap<String, Vec<SubscriptionData>> = HashMap::new();
        let subscriptions = Arc::new(Mutex::new(subscriptions_map));
        let subscriptions_copy = Arc::clone(&subscriptions);

        let reader_fut = async move {
            // TODO: reconnect
            loop {
                let data = reader.next().await;
                if let Err(err) = WsManager::parse_data(data, &subscriptions_copy).await {
                    error!("{err}");
                }
            }
        };
        spawn(reader_fut);

        Ok(WsManager {
            writer,
            subscriptions,
            subscription_id: 0,
            subscription_identifiers: HashMap::new(),
        })
    }

    pub(crate) async fn add_subscription(
        &mut self,
        identifier: String,
        sending_channel: mpsc::UnboundedSender<String>,
    ) -> Result<u32> {
        let mut subscriptions = self.subscriptions.lock().await;

        let entry = subscriptions
            .entry(identifier.clone())
            .or_insert(Vec::new());

        if identifier.eq("userEvents") && !entry.is_empty() {
            return Err(Error::MultipleUserEvents);
        }

        if entry.is_empty() {
            let payload = serde_json::to_string(&WebsocketData {
                method: "subscribe",
                subscription: &serde_json::from_str::<serde_json::Value>(&identifier)
                    .map_err(|e| Error::JsonParse(e.to_string()))?,
            })
            .map_err(|e| Error::JsonParse(e.to_string()))?;

            self.writer
                .send(Message::Text(payload))
                .await
                .map_err(|e| Error::Websocket(e.to_string()))?;
        }

        let subscription_id = self.subscription_id;
        self.subscription_identifiers
            .insert(subscription_id, identifier.clone());
        entry.push(SubscriptionData {
            sending_channel,
            subscription_id,
        });

        self.subscription_id += 1;

        Ok(subscription_id)
    }

    pub(crate) async fn remove_subscription(&mut self, subscription_id: u32) -> Result<()> {
        let identifier = self
            .subscription_identifiers
            .get(&subscription_id)
            .ok_or(Error::SubscriptionNotFound)?
            .clone();

        self.subscription_identifiers.remove(&subscription_id);

        let mut subscriptions = self.subscriptions.lock().await;

        let subscriptions = subscriptions
            .get_mut(&identifier)
            .ok_or(Error::SubscriptionNotFound)?;
        let index = subscriptions
            .iter()
            .position(|subscription_data| subscription_data.subscription_id == subscription_id)
            .ok_or(Error::SubscriptionNotFound)?;
        subscriptions.remove(index);

        if subscriptions.is_empty() {
            let payload = serde_json::to_string(&WebsocketData {
                method: "unsubscribe",
                subscription: &serde_json::from_str::<serde_json::Value>(&identifier)
                    .map_err(|e| Error::JsonParse(e.to_string()))?,
            })
            .map_err(|e| Error::JsonParse(e.to_string()))?;

            self.writer
                .send(Message::Text(payload))
                .await
                .map_err(|e| Error::Websocket(e.to_string()))?;
        }
        Ok(())
    }
}
