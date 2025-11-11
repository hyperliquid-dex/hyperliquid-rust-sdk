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

    // Ensure we're using the actual user's wallet, not an agent
    if address != exchange_client.wallet.address() {
        panic!("Agents do not have permission to convert to multi-sig user");
    }

    // Addresses that will be authorized to sign for the multi-sig account
    let authorized_user_1: Address = "0x0000000000000000000000000000000000000000"
        .parse()
        .unwrap();
    let authorized_user_2: Address = "0x0000000000000000000000000000000000000001"
        .parse()
        .unwrap();

    // Threshold: minimum number of signatures required to execute any transaction
    // This matches the Python example where threshold is 1
    let threshold = 1;

    info!("=== Convert to Multi-Sig User Example ===");
    info!("Current user address: {}", address);
    info!("Connected to: {:?}", exchange_client.http_client.base_url);
    info!("");
    info!("Configuration:");
    info!("  Authorized user 1: {}", authorized_user_1);
    info!("  Authorized user 2: {}", authorized_user_2);
    info!("  Threshold: {}", threshold);
    info!("");

    // Step 1: Convert the user to a multi-sig account
    info!("Step 1: Converting to multi-sig account...");
    match exchange_client.convert_to_multi_sig(threshold, None).await {
        Ok(response) => {
            info!("Convert to multi-sig response: {:?}", response);
            info!("Successfully converted to multi-sig!");
        }
        Err(e) => {
            info!("Convert to multi-sig failed (this is expected if already converted or on testnet): {}", e);
        }
    }

    // Step 2: Add authorized addresses
    info!("Step 2: Adding authorized addresses...");
    match exchange_client
        .update_multi_sig_addresses(
            vec![authorized_user_1, authorized_user_2],
            vec![], // No addresses to remove
            None,
        )
        .await
    {
        Ok(response) => {
            info!("Update multi-sig addresses response: {:?}", response);
            info!("Successfully added authorized addresses!");
        }
        Err(e) => {
            info!("Update multi-sig addresses failed: {}", e);
        }
    }

    info!("");
    info!("Multi-sig setup complete!");
    info!("Now you can use the multi-sig methods with the authorized wallets:");
    info!("- multi_sig_order()");
    info!("- multi_sig_usdc_transfer()");
    info!("- multi_sig_spot_transfer()");
    info!("");
    info!("IMPORTANT: After converting to multi-sig:");
    info!("1. The account can only be controlled by the authorized addresses");
    info!(
        "2. You need {} signatures to execute any transaction",
        threshold
    );
    info!("3. Make sure you have access to the authorized private keys!");
    info!("4. This is a one-way conversion - test on testnet first!");

    info!("Example completed - multi-sig conversion functionality demonstrated");
}
