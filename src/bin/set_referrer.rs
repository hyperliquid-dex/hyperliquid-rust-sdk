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

    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    let code = "TESTNET".to_string();

    let res = exchange_client.set_referrer(code, None).await;

    if let Ok(res) = res {
        info!("Exchange response: {res:#?}");
    } else {
        info!("Got error: {:#?}", res.err().unwrap());
    }
}
