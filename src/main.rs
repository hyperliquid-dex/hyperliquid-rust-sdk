use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::ExchangeClient;
#[tokio::main]
async fn main() {
    let wallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse::<LocalWallet>()
        .unwrap();
    let exchange_client = ExchangeClient::new(
        None,
        wallet,
        Some("https://api.hyperliquid-testnet.xyz"),
        None,
        None,
    )
    .await
    .unwrap();
    let ret = exchange_client
        .usdc_transfer("1", "0x0D1d9635D0640821d15e323ac8AdADfA9c111414")
        .await;
    println!("{:?}", ret);
}
