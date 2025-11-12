use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;
use std::env;

#[tokio::main]
async fn main() {
    env_logger::init();

    let private_key = env::var("PRIVATE_KEY")
        .expect("PRIVATE_KEY environment variable must be set");

    let wallet: LocalWallet = private_key.parse().unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    let destination = "0x5683e89582c45366Ecd224A77853C8A5dE9b6e79";
    let source_dex = "flxn"; // empty string for default USDC perp DEX
    let destination_dex = "spot"; // transfer to spot
    let token = "USDH:0x471fd4480bb9943a1fe080ab0d4ff36c";
    let amount = "1";
    let from_sub_account = ""; // empty string if not from a subaccount

    let res = exchange_client
        .send_asset(
            destination,
            source_dex,
            destination_dex,
            token,
            amount,
            from_sub_account,
            None,
        )
        .await
        .unwrap();
    info!("Send asset result: {res:?}");
}
