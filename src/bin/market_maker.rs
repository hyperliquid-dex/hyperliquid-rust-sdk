use alloy::primitives::{Address, U256};
use hyperliquid_rust_sdk::{
    BaseUrl, BuilderInfo, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient, LocalWallet,
};
use log::info;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
    let wallet = priv_key.parse::<LocalWallet>().unwrap();

    let exchange_client = ExchangeClient::new(BaseUrl::Testnet.get_url());

    let builder_info = BuilderInfo {
        builder: "test".to_string(),
        fee: 1,
    };

    // Place a limit buy order
    let buy_order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 2000.0,
        sz: 0.1,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let response = exchange_client
        .order(buy_order, Some(builder_info.clone()))
        .await
        .unwrap();
    info!("Buy order response: {response:?}");

    tokio::time::sleep(Duration::from_secs(5)).await;

    // Place a limit sell order
    let sell_order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: false,
        reduce_only: false,
        limit_px: 2100.0,
        sz: 0.1,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let response = exchange_client
        .order(sell_order, Some(builder_info))
        .await
        .unwrap();
    info!("Sell order response: {response:?}");
}
