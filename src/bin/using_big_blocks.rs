use alloy::signers::local::PrivateKeySigner;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: PrivateKeySigner =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let exchange_client =
        ExchangeClient::new(None, wallet.clone(), Some(BaseUrl::Testnet), None, None)
            .await
            .unwrap();

    let res = exchange_client
        .enable_big_blocks(false, Some(&wallet))
        .await
        .unwrap();
    info!("enable big blocks : {res:?}");
}
