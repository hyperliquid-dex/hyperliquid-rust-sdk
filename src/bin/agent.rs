use alloy::{
    primitives::{Address, U256},
    signers::local::PrivateKeySigner,
};
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
    let wallet = priv_key.parse::<PrivateKeySigner>().unwrap();

    let exchange_client = ExchangeClient::new(BaseUrl::Testnet.get_url());

    let agent = "0x1234567890123456789012345678901234567890"
        .parse::<Address>()
        .unwrap();

    info!("Approving agent {}", agent);

    let res = exchange_client
        .approve_agent(agent, "Testnet".to_string())
        .await;
    match res {
        Ok(_) => info!("Successfully approved agent"),
        Err(e) => eprintln!("Failed to approve agent: {}", e),
    }
}
