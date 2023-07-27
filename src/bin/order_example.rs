use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::ExchangeClient;
use hyperliquid_rust_sdk::{ClientLimit, ClientOrderRequest, ClientOrderType};

#[tokio::main]
async fn main() {
    let wallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse::<LocalWallet>()
        .unwrap();

    let exchange_client = ExchangeClient::new(
        None,
        wallet,
        Some("https://api.hyperliquid-testnet.xyz"),
        None,
        None,
    )
    .await
    .unwrap();

    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1800.0,
        sz: 0.01,
        order_type: ClientOrderType::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    println!("{}", exchange_client.order(order).await.unwrap());
}
