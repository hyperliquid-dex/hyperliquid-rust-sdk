use crate::{
    exchange::Actions,
    helpers::next_nonce,
    signature::sign_l1_action,
    BaseUrl, BulkCancelCloid, BulkOrder, Error, ExchangeResponseStatus,
};
use ethers::{
    signers::LocalWallet,
    types::{H160, H256},
};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    net::TcpStream,
    spawn,
    sync::{Mutex, oneshot},
    time::timeout,
};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::protocol::{self, WebSocketConfig},
    MaybeTlsStream, WebSocketStream,
};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WsPostRequest<T> {
    method: String,
    id: u64,
    request: WsRequestData<T>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WsRequestData<T> {
    #[serde(rename = "type")]
    request_type: String,
    payload: T,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WsPostResponse {
    channel: String,
    data: WsResponseData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WsResponseData {
    id: u64,
    response: WsResponse,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum WsResponse {
    Action { payload: ExchangeResponseStatus },
    Error { payload: String },
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WsExchangePayload {
    action: serde_json::Value,
    signature: WsSignature,
    nonce: u64,
    vault_address: Option<H160>,
}

#[derive(Serialize, Debug)]
struct WsSignature {
    r: String,
    s: String,
    v: u8,
}

type ResponseSender = oneshot::Sender<Result<ExchangeResponseStatus, Error>>;

#[derive(Debug)]
pub struct WsPostClient {
    writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, protocol::Message>>>,
    pending_requests: Arc<Mutex<HashMap<u64, ResponseSender>>>,
    request_id_counter: AtomicU64,
}

impl WsPostClient {
    pub async fn new(base_url: BaseUrl) -> Result<Self, Error> {
        let url = match base_url {
            BaseUrl::Mainnet => "wss://api.hyperliquid.xyz/ws",
            BaseUrl::Testnet => "wss://api.hyperliquid-testnet.xyz/ws",
            BaseUrl::Localhost => "ws://localhost:3001/ws",
        };

        let (ws_stream, _) = connect_async_with_config(
            url,
            Some(create_optimized_websocket_config()),
            true,
        )
        .await
        .map_err(|e| Error::Websocket(e.to_string()))?;

        let (writer, mut reader) = ws_stream.split();
        let writer = Arc::new(Mutex::new(writer));
        let pending_requests: Arc<Mutex<HashMap<u64, ResponseSender>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Spawn reader task to handle responses
        let pending_requests_clone = pending_requests.clone();
        spawn(async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(protocol::Message::Text(text)) => {
                        if let Err(e) = Self::handle_response(text.to_string(), &pending_requests_clone).await {
                            error!("Error handling websocket response: {}", e);
                        }
                    }
                    Ok(protocol::Message::Pong(_)) => {
                        debug!("Received pong");
                    }
                    Ok(_) => {
                        debug!("Received non-text message");
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        // Notify all pending requests about the error
                        let mut pending = pending_requests_clone.lock().await;
                        for (_, sender) in pending.drain() {
                            let _ = sender.send(Err(Error::Websocket(e.to_string())));
                        }
                        break;
                    }
                }
            }
        });

        Ok(Self {
            writer,
            pending_requests,
            request_id_counter: AtomicU64::new(1),
        })
    }

    async fn handle_response(
        text: String,
        pending_requests: &Arc<Mutex<HashMap<u64, ResponseSender>>>,
    ) -> Result<(), Error> {
        // First try to parse as a proper response
        if let Ok(response) = serde_json::from_str::<WsPostResponse>(&text) {
            if response.channel == "post" {
                let mut pending = pending_requests.lock().await;
                if let Some(sender) = pending.remove(&response.data.id) {
                    let result = match response.data.response {
                        WsResponse::Action { payload } => Ok(payload),
                        WsResponse::Error { payload } => Err(Error::GenericRequest(payload)),
                    };
                    let _ = sender.send(result);
                }
            }
            return Ok(());
        }

        // If that fails, it might be an error string - log it
        error!("Received non-standard response: {}", text);
        
        // For now, we can't correlate this to a specific request, so we'll ignore it
        // In a production system, you might want to handle this differently
        Ok(())
    }

    async fn send_request<T: Serialize>(
        &self,
        payload: T,
        timeout_duration: Duration,
    ) -> Result<ExchangeResponseStatus, Error> {
        let request_id = self.request_id_counter.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();

        // Store the response sender
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(request_id, tx);
        }

        // Create and send the request
        let request = WsPostRequest {
            method: "post".to_string(),
            id: request_id,
            request: WsRequestData {
                request_type: "action".to_string(),
                payload,
            },
        };

        let message_text =
            serde_json::to_string(&request).map_err(|e| Error::JsonParse(e.to_string()))?;

        {
            let mut writer = self.writer.lock().await;
            writer
                .send(protocol::Message::Text(message_text.into()))
                .await
                .map_err(|e| Error::Websocket(e.to_string()))?;
        }

        // Wait for response with timeout
        match timeout(timeout_duration, rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(Error::GenericRequest("Response channel closed".to_string())),
            Err(_) => {
                // Remove the pending request on timeout
                let mut pending = self.pending_requests.lock().await;
                pending.remove(&request_id);
                Err(Error::GenericRequest("Request timeout".to_string()))
            }
        }
    }

    pub async fn bulk_order(
        &self,
        action: BulkOrder,
        wallet: &LocalWallet,
        is_mainnet: bool,
        vault_address: Option<H160>,
    ) -> Result<ExchangeResponseStatus, Error> {
        let timestamp = next_nonce();
        let full_action = Actions::Order(action);
        let connection_id = self.calculate_action_hash(&full_action, timestamp, vault_address)?;
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        let r = format!("0x{:x}", signature.r);
        let s = format!("0x{:x}", signature.s);
        let v = signature.v as u8;

        let payload = WsExchangePayload {
            action: serde_json::to_value(&full_action)
                .map_err(|e| Error::JsonParse(e.to_string()))?,
            signature: WsSignature { r, s, v },
            nonce: timestamp,
            vault_address,
        };

        self.send_request(payload, Duration::from_secs(15)).await
    }

    pub async fn bulk_cancel_by_cloid(
        &self,
        action: BulkCancelCloid,
        wallet: &LocalWallet,
        is_mainnet: bool,
        vault_address: Option<H160>,
    ) -> Result<ExchangeResponseStatus, Error> {
        let timestamp = next_nonce();
        let full_action = Actions::CancelByCloid(action);
        let connection_id = self.calculate_action_hash(&full_action, timestamp, vault_address)?;
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        let r = format!("0x{:x}", signature.r);
        let s = format!("0x{:x}", signature.s);
        let v = signature.v as u8;

        let payload = WsExchangePayload {
            action: serde_json::to_value(&full_action)
                .map_err(|e| Error::JsonParse(e.to_string()))?,
            signature: WsSignature { r, s, v },
            nonce: timestamp,
            vault_address,
        };

        self.send_request(payload, Duration::from_secs(15)).await
    }

    fn calculate_action_hash<T: Serialize>(
        &self,
        action: &T,
        timestamp: u64,
        vault_address: Option<H160>,
    ) -> Result<H256, Error> {
        let mut bytes =
            rmp_serde::to_vec_named(action).map_err(|e| Error::RmpParse(e.to_string()))?;
        bytes.extend(timestamp.to_be_bytes());
        if let Some(vault_address) = vault_address {
            bytes.push(1);
            bytes.extend(vault_address.to_fixed_bytes());
        } else {
            bytes.push(0);
        }
        Ok(H256(ethers::utils::keccak256(bytes)))
    }
}

/// Create optimized WebSocket configuration for low-latency trading
fn create_optimized_websocket_config() -> WebSocketConfig {
    let mut config = WebSocketConfig::default();
    
    config.read_buffer_size = 64 * 1024;
    config.write_buffer_size = 0;
    config.max_write_buffer_size = 512 * 1024;
    config.max_message_size = None;
    config.max_frame_size = Some(128 * 1024);
    config.accept_unmasked_frames = false;
    
    config
}