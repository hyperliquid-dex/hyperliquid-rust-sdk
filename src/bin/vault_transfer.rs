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

    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    let usd = 5_000_000; // at least 5 USD
    let is_deposit = true;

    let res = exchange_client
        .vault_transfer(
            is_deposit,
            usd,
            Some(
                "0x1962905b0a2d0ce7907ae1a0d17f3e4a1f63dfb7"
                    .parse()
                    .unwrap(),
            ),
            None,
        )
        .await
        .unwrap();
    info!("Vault transfer result: {res:?}");
}
