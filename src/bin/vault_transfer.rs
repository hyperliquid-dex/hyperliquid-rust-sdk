use alloy_primitives::{Address, U256};
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

    let vault_address = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414".parse::<Address>().unwrap();
    let amount = "1"; // 1 USD

    info!("Depositing {} USD to vault {}", amount, vault_address);

    let response = exchange_client
        .vault_transfer(vault_address, true, amount.to_string(), "Testnet".to_string())
        .await
        .unwrap();
    info!("Vault deposit response: {response:?}");
}
