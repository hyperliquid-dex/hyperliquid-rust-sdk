use alloy::primitives::{Address, U256};
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, LocalWallet};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
    let wallet = priv_key.parse::<LocalWallet>().unwrap();

    let exchange_client = ExchangeClient::new(BaseUrl::Testnet.get_url());

    let code = "TESTNET".to_string();

    let res = exchange_client.set_referrer(code).await;
    match res {
        Ok(_) => info!("Successfully set referrer code"),
        Err(e) => eprintln!("Failed to set referrer code: {}", e),
    }
}
