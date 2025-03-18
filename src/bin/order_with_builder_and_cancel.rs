use alloy::primitives::{Address, U256};
use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, BuilderInfo, ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest,
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

    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1800.0,
        sz: 0.1,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let builder = BuilderInfo {
        builder: "0x1962905b0a2d0ce7907ae1a0d17f3e4a1f63dfb7".to_string(),
        fee: 1,
    };

    info!("Placing order with builder: {:?}", order);
    let res = exchange_client
        .order(order.clone(), Some(builder.clone()))
        .await
        .unwrap();
    info!("Order result: {:?}", res);

    let response = match res {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("error with exchange response: {e}"),
    };

    let oid = if let Some(data) = response.data {
        if !data.statuses.is_empty() {
            match data.statuses[0].clone() {
                ExchangeDataStatus::Filled(order) => order.oid,
                ExchangeDataStatus::Resting(order) => order.oid,
                ExchangeDataStatus::Error(e) => panic!("error with order: {e}"),
                _ => unreachable!(),
            }
        } else {
            panic!("no order status");
        }
    } else {
        panic!("no order data");
    };

    // So you can see the order before it's cancelled
    sleep(Duration::from_secs(10));

    let cancel = ClientCancelRequest {
        asset: order.asset,
        oid,
    };

    info!("Cancelling order");
    let res = exchange_client.cancel(cancel, Some(builder)).await.unwrap();
    info!("Cancel result: {:?}", res);
}
