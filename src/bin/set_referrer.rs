/*
This is an example of setting a refferal code for a wallet.
*/
use ethers::signers::LocalWallet;

use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Add a testnet key which has deposited some funds on HL testnet
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    let ex = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .expect("Couldn't get the exchange client");

    let code = "TESTNET".to_string();

    let res = ex.set_referrer(code, None).await;

    if let Ok(res) = res {
        info!("Exchange response: {res:#?}");
    } else {
        info!("Got error: {:#?}", res.err().unwrap());
    }
}
