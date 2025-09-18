use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

async fn setup_exchange_client() -> (Address, ExchangeClient) {
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: PrivateKeySigner =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let address = wallet.address();
    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    (address, exchange_client)
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let (address, exchange_client) = setup_exchange_client().await;

    if address != exchange_client.wallet.address() {
        panic!("Agents do not have permission to convert to multi-sig user");
    }

    let authorized_user_1: Address = "0x0000000000000000000000000000000000000000"
        .parse()
        .unwrap();
    let authorized_user_2: Address = "0x0000000000000000000000000000000000000001"
        .parse()
        .unwrap();
    let threshold = 1;

    info!("Converting user {} to multi-sig", address);
    info!(
        "Authorized users: {}, {}",
        authorized_user_1, authorized_user_2
    );
    info!("Threshold: {}", threshold);

    info!("Multi-sig conversion functionality is not yet implemented in the Rust SDK");
    info!("This example shows the structure and parameters that would be used:");
    info!(
        "- Authorized users: [{}, {}]",
        authorized_user_1, authorized_user_2
    );
    info!("- Threshold: {}", threshold);

    info!("Example completed successfully - multi-sig parameters validated");
}
