use alloy::signers::local::PrivateKeySigner;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, InfoClient};
use log::info;

#[tokio::main]
async fn main() {
    // Example assumes you already have a position on ETH so you can update margin
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: PrivateKeySigner =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let address = wallet.address();
    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();
    let info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();

    let response = exchange_client
        .update_leverage(5, "ETH", false, None)
        .await
        .unwrap();
    info!("Update leverage response: {response:?}");

    let response = exchange_client
        .update_isolated_margin(1.0, "ETH", None)
        .await
        .unwrap();

    info!("Update isolated margin response: {response:?}");

    let user_state = info_client.user_state(address).await.unwrap();
    info!("User state: {user_state:?}");
}
