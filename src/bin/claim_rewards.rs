use alloy::signers::local::PrivateKeySigner;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, ExchangeResponseStatus};
use log::info;

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

    let response = exchange_client.claim_rewards(None).await.unwrap();

    match response {
        ExchangeResponseStatus::Ok(exchange_response) => {
            info!("Rewards claimed successfully: {exchange_response:?}");
        }
        ExchangeResponseStatus::Err(e) => {
            info!("Failed to claim rewards: {e}");
        }
    }
}
