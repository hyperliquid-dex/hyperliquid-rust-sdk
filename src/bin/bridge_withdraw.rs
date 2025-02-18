use alloy::primitives::{Address, U256};
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, LocalWallet};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    let exchange_client = ExchangeClient::new(BaseUrl::Testnet.get_url());

    let amount = "5"; // 5 USD
    let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";

    let amount = amount.parse::<U256>().unwrap();
    let destination = destination.parse::<Address>().unwrap();

    exchange_client
        .withdraw(destination, amount, "Testnet".to_string())
        .await
        .unwrap();
    info!("Withdraw completed");
}
