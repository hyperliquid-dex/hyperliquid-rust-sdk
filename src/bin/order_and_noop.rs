use ethers::signers::LocalWallet;
use log::info;
use std::sync::Arc;
use tokio;

use hyperliquid_rust_sdk::{BaseUrl, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: LocalWallet = "fake".parse().unwrap();

    let mut exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // Initialize the WebSocket client to send low-latency requests
    exchange_client.init_ws_post_client().await.unwrap();

    // Wrap the client in an Arc to allow safe, shared access across multiple tasks
    let exchange_client = Arc::new(exchange_client);

    // Define the order we intend to place
    let cloid = Uuid::new_v4();
    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1800.0,
        sz: 0.01,
        cloid: Some(cloid),
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    // 1. Prepare the order request without sending it. This calculates all signatures
    //    and crucially, assigns a nonce to the transaction.
    let prepared_order = exchange_client
        .prepare_bulk_order_ws(vec![order], None)
        .unwrap();

    // 2. Extract the nonce that was used to prepare the order.
    let nonce = prepared_order.nonce;
    info!("Using nonce: {nonce} to race an order and a noop transaction.");

    // 3. Concurrently send both the prepared order and a new noop transaction
    //    using the SAME nonce. The server will only accept the first one it sees.

    // Clone the Arc for the first task
    let client_for_order = Arc::clone(&exchange_client);
    let order_task = tokio::spawn(async move {
        let result = client_for_order
            .send_prepared_bulk_order_ws(prepared_order)
            .await;
        info!("Order send result: {:?}", result);
    });

    // Clone the Arc for the second task
    let client_for_noop = Arc::clone(&exchange_client);
    let noop_task = tokio::spawn(async move {
        // Use the WebSocket noop for the lowest latency race
        let result = client_for_noop.noop_ws(nonce, None).await;
        info!("No-op send result: {:?}", result);
    });

    // Wait for both tasks to complete
    let _ = tokio::join!(order_task, noop_task);

    info!("Both tasks completed. Check logs to see which one succeeded and which one failed due to the nonce.");
}
