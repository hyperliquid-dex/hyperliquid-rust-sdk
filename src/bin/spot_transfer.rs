use alloy_primitives::{Address, U256};
use alloy_signer_local::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
    let wallet = priv_key.parse::<LocalWallet>().unwrap();

    let exchange_client = ExchangeClient::new(BaseUrl::Testnet.get_url());

    let amount = "1";
    let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";
    let token = "ETH";

    let amount = amount.parse::<U256>().unwrap();
    let destination = destination.parse::<Address>().unwrap();

    info!("Sending {} {} to {}", amount, token, destination);

    exchange_client
        .spot_send(destination, token.to_string(), amount, "Testnet".to_string())
        .await
        .unwrap();
}
