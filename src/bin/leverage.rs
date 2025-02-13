use alloy_primitives::U256;
use alloy_signer_local::PrivateKeySigner;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
    let wallet = priv_key.parse::<PrivateKeySigner>().unwrap();

    let exchange_client = ExchangeClient::new(BaseUrl::Testnet.get_url());

    // TODO: Implement leverage functionality using the new API
    info!("Leverage functionality not yet implemented");
}
