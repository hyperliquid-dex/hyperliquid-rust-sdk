use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    let exchange_client =
        ExchangeClient::new(None, wallet.clone(), Some(BaseUrl::Testnet), None, None)
            .await
            .unwrap();

    let max_fee_rate = "0.1%";
    let builder = "0x1ab189B7801140900C711E458212F9c76F8dAC79".to_lowercase();

    let resp = exchange_client
        .approve_builder_fee(builder.to_string(), max_fee_rate.to_string(), Some(&wallet))
        .await;
    info!("resp: {resp:#?}");
}
