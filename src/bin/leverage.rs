use ethers::signers::{LocalWallet, Signer};
use hyperliquid_rust_sdk::{ExchangeClient, InfoClient, TESTNET_API_URL};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Example assumes you already have a position on ETH so you can update margin
    let wallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse::<LocalWallet>()
        .unwrap();

    let address = wallet.address();
    let exchange_client = ExchangeClient::new(None, wallet, Some(TESTNET_API_URL), None, None)
        .await
        .unwrap();
    let info_client = InfoClient::new(None, Some(TESTNET_API_URL)).await.unwrap();

    let response = exchange_client
        .update_leverage(5, "ETH", false)
        .await
        .unwrap();
    info!("Update leverage response: {response:?}");

    let response = exchange_client
        .update_isolated_margin(1.0, "ETH")
        .await
        .unwrap();

    info!("Update isolated margin response: {response:?}");

    let user_state = info_client.user_state(address).await.unwrap();
    info!("User state: {user_state:?}");
}
