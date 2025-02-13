use alloy_primitives::U256;
use alloy_signer_local::PrivateKeySigner;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, ClientOrder, ClientOrderRequest, BuilderInfo, ClientLimit};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
    let wallet = priv_key.parse::<PrivateKeySigner>().unwrap();

    let exchange_client = ExchangeClient::new(BaseUrl::Testnet.get_url());

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

    let builder = Some(BuilderInfo {
        builder: "0x1234567890123456789012345678901234567890".to_string(),
        fee: 1,
    });

    let response = exchange_client.order(order, builder).await.unwrap();
    info!("Order placed: {response:?}");
}
