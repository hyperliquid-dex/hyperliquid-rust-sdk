use crate::{
    exchange::Actions, helpers::next_nonce, signature::sign_l1_action, BaseUrl, BulkCancelCloid,
    BulkOrder, Error, ExchangeResponseStatus,
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
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    net::TcpStream,
    spawn,
    sync::{oneshot, Mutex},
    time::{sleep, timeout, Instant},
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
struct WsPongResponse {
    channel: String,
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

#[derive(Serialize)]
struct Ping {
    method: &'static str,
}

type ResponseSender = oneshot::Sender<Result<ExchangeResponseStatus, Error>>;

/// Timing statistics for a specific operation
#[derive(Debug, Clone, Copy, Default)]
pub struct TimingStats {
    pub avg_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    count: u64,
    sum_ms: f64,
}

impl TimingStats {
    fn update(&mut self, duration_ms: f64) {
        self.count += 1;
        self.sum_ms += duration_ms;
        self.avg_ms = self.sum_ms / self.count as f64;
        self.min_ms = self.min_ms.min(duration_ms);
        self.max_ms = self.max_ms.max(duration_ms);
    }
}

/// Performance metrics for bulk order operations
#[derive(Debug, Clone, Copy, Default)]
pub struct BulkOrderMetrics {
    pub compute_time: TimingStats,
    pub send_time: TimingStats,
}

/// Performance metrics for bulk cancel operations
#[derive(Debug, Clone, Copy, Default)]
pub struct BulkCancelMetrics {
    pub compute_time: TimingStats,
    pub send_time: TimingStats,
}

#[derive(Debug)]
struct PerformanceMetrics {
    bulk_order: Mutex<BulkOrderMetrics>,
    bulk_cancel: Mutex<BulkCancelMetrics>,
}

#[derive(Debug)]
pub struct WsPostClient {
    writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, protocol::Message>>>,
    pending_requests: Arc<Mutex<HashMap<u64, ResponseSender>>>,
    request_id_counter: AtomicU64,
    stop_flag: Arc<AtomicBool>,
    performance_logging: bool,
    metrics: Option<PerformanceMetrics>,
}

impl WsPostClient {
    const SEND_PING_INTERVAL: u64 = 50;

    pub async fn new(base_url: BaseUrl) -> Result<Self, Error> {
        Self::with_performance_logging(base_url, false).await
    }

    pub async fn with_performance_logging(
        base_url: BaseUrl,
        performance_logging: bool,
    ) -> Result<Self, Error> {
        let url = match base_url {
            BaseUrl::Mainnet => "wss://api.hyperliquid.xyz/ws",
            BaseUrl::Testnet => "wss://api.hyperliquid-testnet.xyz/ws",
            BaseUrl::Localhost => "ws://localhost:3001/ws",
        };

        let (ws_stream, _) =
            connect_async_with_config(url, Some(create_optimized_websocket_config()), true)
                .await
                .map_err(|e| Error::Websocket(e.to_string()))?;

        let (writer, mut reader) = ws_stream.split();
        let writer = Arc::new(Mutex::new(writer));
        let pending_requests: Arc<Mutex<HashMap<u64, ResponseSender>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let stop_flag = Arc::new(AtomicBool::new(false));

        // Spawn reader task to handle responses
        let pending_requests_clone = pending_requests.clone();
        let stop_flag_clone = stop_flag.clone();
        spawn(async move {
            while !stop_flag_clone.load(Ordering::Relaxed) {
                if let Some(msg) = reader.next().await {
                    match msg {
                        Ok(protocol::Message::Text(text)) => {
                            if let Err(e) =
                                Self::handle_response(text.to_string(), &pending_requests_clone)
                                    .await
                            {
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
                } else {
                    error!("WebSocket connection closed");
                    break;
                }
            }
        });

        // Spawn ping task to keep connection alive
        {
            let stop_flag_clone = stop_flag.clone();
            let writer_clone = writer.clone();
            spawn(async move {
                while !stop_flag_clone.load(Ordering::Relaxed) {
                    match serde_json::to_string(&Ping { method: "ping" }) {
                        Ok(payload) => {
                            let mut writer = writer_clone.lock().await;
                            if let Err(err) =
                                writer.send(protocol::Message::Text(payload.into())).await
                            {
                                error!("Error pinging server: {}", err);
                            }
                        }
                        Err(err) => error!("Error serializing ping message: {}", err),
                    }
                    sleep(Duration::from_secs(Self::SEND_PING_INTERVAL)).await;
                }
            });
        }

        let metrics = if performance_logging {
            Some(PerformanceMetrics {
                bulk_order: Mutex::new(BulkOrderMetrics::default()),
                bulk_cancel: Mutex::new(BulkCancelMetrics::default()),
            })
        } else {
            None
        };

        Ok(Self {
            writer,
            pending_requests,
            request_id_counter: AtomicU64::new(1),
            stop_flag,
            performance_logging,
            metrics,
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

        // Then try to parse as a pong response
        if let Ok(pong_response) = serde_json::from_str::<WsPongResponse>(&text) {
            if pong_response.channel == "pong" {
                debug!("Received pong from server");
                return Ok(());
            }
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

    /// Private core function for sending a bulk order.
    /// Returns the response and the nonce used.
    async fn _send_bulk_order(
        &self,
        action: BulkOrder,
        wallet: &LocalWallet,
        is_mainnet: bool,
        vault_address: Option<H160>,
    ) -> Result<(ExchangeResponseStatus, u64), Error> {
        let compute_start = if self.performance_logging {
            Some(Instant::now())
        } else {
            None
        };

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

        let compute_duration = compute_start.map(|start| start.elapsed().as_secs_f64() * 1000.0);

        let send_start = if self.performance_logging {
            Some(Instant::now())
        } else {
            None
        };

        let result = self.send_request(payload, Duration::from_secs(15)).await;

        if self.performance_logging {
            if let Some(compute_ms) = compute_duration {
                if let Some(send_start) = send_start {
                    let send_ms = send_start.elapsed().as_secs_f64() * 1000.0;
                    if let Some(metrics) = &self.metrics {
                        let mut bulk_order_metrics = metrics.bulk_order.lock().await;
                        bulk_order_metrics.compute_time.update(compute_ms);
                        bulk_order_metrics.send_time.update(send_ms);
                    }
                }
            }
        }

        result.map(|res| (res, timestamp))
    }

    /// The original bulk_order function, now a thin wrapper.
    pub async fn bulk_order(
        &self,
        action: BulkOrder,
        wallet: &LocalWallet,
        is_mainnet: bool,
        vault_address: Option<H160>,
    ) -> Result<ExchangeResponseStatus, Error> {
        let (response, _nonce) = self
            ._send_bulk_order(action, wallet, is_mainnet, vault_address)
            .await?;
        Ok(response)
    }

    /// New function that returns the nonce along with the response.
    pub async fn bulk_order_with_nonce(
        &self,
        action: BulkOrder,
        wallet: &LocalWallet,
        is_mainnet: bool,
        vault_address: Option<H160>,
    ) -> Result<(ExchangeResponseStatus, u64), Error> {
        self._send_bulk_order(action, wallet, is_mainnet, vault_address)
            .await
    }

    pub async fn bulk_cancel_by_cloid(
        &self,
        action: BulkCancelCloid,
        wallet: &LocalWallet,
        is_mainnet: bool,
        vault_address: Option<H160>,
    ) -> Result<ExchangeResponseStatus, Error> {
        let compute_start = if self.performance_logging {
            Some(Instant::now())
        } else {
            None
        };

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

        let compute_duration = compute_start.map(|start| start.elapsed().as_secs_f64() * 1000.0);

        let send_start = if self.performance_logging {
            Some(Instant::now())
        } else {
            None
        };

        let result = self.send_request(payload, Duration::from_secs(15)).await;

        if self.performance_logging {
            if let Some(compute_ms) = compute_duration {
                if let Some(send_start) = send_start {
                    let send_ms = send_start.elapsed().as_secs_f64() * 1000.0;
                    if let Some(metrics) = &self.metrics {
                        let mut bulk_cancel_metrics = metrics.bulk_cancel.lock().await;
                        bulk_cancel_metrics.compute_time.update(compute_ms);
                        bulk_cancel_metrics.send_time.update(send_ms);
                    }
                }
            }
        }

        result
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

    pub async fn noop(
        &self,
        nonce: u64,
        wallet: &LocalWallet,
        is_mainnet: bool,
        vault_address: Option<H160>,
    ) -> Result<ExchangeResponseStatus, Error> {
        let full_action = Actions::Noop;
        let connection_id = self.calculate_action_hash(&full_action, nonce, vault_address)?;
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        let r = format!("0x{:x}", signature.r);
        let s = format!("0x{:x}", signature.s);
        let v = signature.v as u8;

        let payload = WsExchangePayload {
            action: serde_json::to_value(&full_action)
                .map_err(|e| Error::JsonParse(e.to_string()))?,
            signature: WsSignature { r, s, v },
            nonce, // Use the provided nonce
            vault_address,
        };

        self.send_request(payload, Duration::from_secs(15)).await
    }

    /// Get performance metrics for bulk order operations
    /// Returns None if performance logging is disabled
    pub async fn get_bulk_order_metrics(&self) -> Option<BulkOrderMetrics> {
        if let Some(metrics) = &self.metrics {
            Some(*metrics.bulk_order.lock().await)
        } else {
            None
        }
    }

    /// Get performance metrics for bulk cancel operations
    /// Returns None if performance logging is disabled
    pub async fn get_bulk_cancel_metrics(&self) -> Option<BulkCancelMetrics> {
        if let Some(metrics) = &self.metrics {
            Some(*metrics.bulk_cancel.lock().await)
        } else {
            None
        }
    }

    /// Reset all performance metrics
    /// Does nothing if performance logging is disabled
    pub async fn reset_metrics(&self) {
        if let Some(metrics) = &self.metrics {
            *metrics.bulk_order.lock().await = BulkOrderMetrics::default();
            *metrics.bulk_cancel.lock().await = BulkCancelMetrics::default();
        }
    }
}

impl Drop for WsPostClient {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
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
