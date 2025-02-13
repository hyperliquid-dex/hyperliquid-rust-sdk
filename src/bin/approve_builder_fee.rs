use alloy_primitives::{Address, U256};
use alloy_signer_local::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, BuilderInfo};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
    let wallet = priv_key.parse::<LocalWallet>().unwrap();

    let exchange_client = ExchangeClient::new(BaseUrl::Testnet.get_url());

    let builder = BuilderInfo {
        builder: "0x1ab189B7801140900C711E458212F9c76F8dAC79".to_string(),
        fee: 1,
    };

    info!("Builder info: {:?}", builder);
}
