use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{ExchangeClient, TESTNET_API_URL};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();

    let wallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse::<LocalWallet>()
        .unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(TESTNET_API_URL), None, None)
        .await
        .unwrap();

    let amount = "1"; // 1 USD
    let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";

    let res = exchange_client
        .usdc_transfer(amount, destination)
        .await
        .unwrap();
    info!("Usdc transfer result: {res:?}");
}
