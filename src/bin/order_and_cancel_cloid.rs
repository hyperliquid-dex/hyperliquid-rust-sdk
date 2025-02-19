use alloy::primitives::{Address, U256};
use log::info;
use uuid::Uuid;

use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequestCloid, ClientLimit, ClientOrder, ClientOrderRequest,
    ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus, LocalWallet,
};
use std::{thread::sleep, time::Duration};

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
    let wallet = priv_key.parse::<LocalWallet>().unwrap();

    let exchange_client = ExchangeClient::new(BaseUrl::Testnet.get_url());

    let cloid = Uuid::new_v4();
    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 2000.0,
        sz: 0.1,
        cloid: Some(cloid),
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let response = exchange_client.order(order, None).await.unwrap();
    info!("Order placed: {response:?}");

    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    match status {
        ExchangeDataStatus::Filled(order) => info!("Order filled: {order:?}"),
        ExchangeDataStatus::Resting(order) => info!("Order resting: {order:?}"),
        _ => panic!("Error: {status:?}"),
    };

    // So you can see the order before it's cancelled
    sleep(Duration::from_secs(10));

    let cancel = ClientCancelRequestCloid {
        asset: "ETH".to_string(),
        cloid: cloid.to_string(),
    };

    // This response will return an error if order was filled (since you can't cancel a filled order), otherwise it will cancel the order
    let response = exchange_client.cancel_by_cloid(cancel, None).await.unwrap();
    info!("Order potentially cancelled: {response:?}");
}
