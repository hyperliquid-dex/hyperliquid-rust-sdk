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
    let ret = exchange_client.update_leverage(5, "SOL", true).await;
    println!("{:?}", ret);
}
