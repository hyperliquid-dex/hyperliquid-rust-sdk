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

    let usdc = 1000; // 1000 USDC
    let to_perp = true; // Transfer to perp account

    info!(
        "Transferring {} USDC to {} account",
        usdc,
        if to_perp { "perp" } else { "spot" }
    );

    let amount = U256::from(usdc);

    exchange_client
        .class_transfer(amount, to_perp, "Testnet".to_string())
        .await
        .unwrap();

    info!("Class transfer completed");
}
