use alloy::primitives::{Address, U256};
use hyperliquid_rust_sdk::{
    BaseUrl, ClientLimit, ClientOrder, ClientOrderRequest, ClientTrigger, ExchangeClient,
    ExchangeDataStatus, ExchangeResponseStatus, LocalWallet, MarketCloseParams, MarketOrderParams,
};
use log::info;
use std::{thread::sleep, time::Duration};

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
    let wallet = priv_key.parse::<LocalWallet>().unwrap();

    let exchange_client = ExchangeClient::new(BaseUrl::Testnet.get_url());

    // Open position with a limit order
    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1795.0,
        sz: 0.01,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let response = exchange_client.order(order, None).await.unwrap();
    info!("Order placed: {response:?}");

    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("Error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    match status {
        ExchangeDataStatus::Filled(order) => info!("Order filled: {order:?}"),
        ExchangeDataStatus::Resting(order) => info!("Order resting: {order:?}"),
        _ => panic!("Unexpected status: {status:?}"),
    };

    // Wait for a while before closing the position
    sleep(Duration::from_secs(10));

    // Close position with a market order
    let close_order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: false,     // Opposite direction to close
        reduce_only: true, // This ensures we only reduce/close our position
        limit_px: 1795.0,
        sz: 0.01,
        cloid: None,
        order_type: ClientOrder::Trigger(ClientTrigger {
            is_market: true,
            trigger_px: 1795.0,
            tpsl: "tp".to_string(),
        }),
    };

    let response = exchange_client.order(close_order, None).await.unwrap();
    info!("Close order placed: {response:?}");

    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("Error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    match status {
        ExchangeDataStatus::Filled(order) => info!("Close order filled: {order:?}"),
        ExchangeDataStatus::Resting(order) => info!("Close order resting: {order:?}"),
        _ => panic!("Unexpected status: {status:?}"),
    };
}
