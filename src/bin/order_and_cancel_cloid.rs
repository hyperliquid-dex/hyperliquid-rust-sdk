use alloy::signers::local::PrivateKeySigner;
use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequestCloid, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient,
};
use std::{thread::sleep, time::Duration};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: PrivateKeySigner =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    // Order and Cancel with cloid
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

    let response = exchange_client.order(order, None).await.unwrap();
    info!("Order placed: {response:?}");

    // So you can see the order before it's cancelled
    sleep(Duration::from_secs(10));

    let cancel = ClientCancelRequestCloid {
        asset: "ETH".to_string(),
        cloid,
    };

    // This response will return an error if order was filled (since you can't cancel a filled order), otherwise it will cancel the order
    let response = exchange_client.cancel_by_cloid(cancel, None).await.unwrap();
    info!("Order potentially cancelled: {response:?}");
}
