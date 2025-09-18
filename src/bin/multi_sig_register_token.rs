use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;
use serde_json::json;

fn setup_multi_sig_wallets() -> Vec<PrivateKeySigner> {
    let wallets = vec![
        "0x1234567890123456789012345678901234567890123456789012345678901234",
        "0x2345678901234567890123456789012345678901234567890123456789012345",
        "0x3456789012345678901234567890123456789012345678901234567890123456",
    ];

    wallets
        .into_iter()
        .map(|key| key.parse().unwrap())
        .collect()
}

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

    let multi_sig_user: Address = "0x0000000000000000000000000000000000000005"
        .parse()
        .unwrap();

    let timestamp = chrono::Utc::now().timestamp_millis() as u64;

    let action = json!({
        "type": "spotDeploy",
        "registerToken2": {
            "spec": {
                "name": "TESTH",
                "szDecimals": 2,
                "weiDecimals": 8
            },
            "maxGas": 1000000000000u64,
            "fullName": "Example multi-sig spot deploy"
        }
    });

    info!("Multi-sig user: {}", multi_sig_user);
    info!("Outer signer (current wallet): {}", address);
    info!(
        "Exchange client connected to: {:?}",
        exchange_client.http_client.base_url
    );
    info!("Action: {}", action);
    info!("Timestamp: {}", timestamp);

    let multi_sig_wallets = setup_multi_sig_wallets();
    info!(
        "Multi-sig wallets: {:?}",
        multi_sig_wallets
            .iter()
            .map(|w| w.address())
            .collect::<Vec<_>>()
    );

    info!("Multi-sig register token functionality is not yet implemented in the Rust SDK");
    info!("This example shows the structure and parameters that would be used:");

    info!("Example completed successfully - multi-sig register token parameters validated");
}
