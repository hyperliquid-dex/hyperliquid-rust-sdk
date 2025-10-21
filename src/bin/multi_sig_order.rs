use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use hyperliquid_rust_sdk::{BaseUrl, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient};
use log::info;

fn setup_multi_sig_wallets() -> Vec<PrivateKeySigner> {
    // These are example private keys - in production, these would be the authorized
    // user wallets that have permission to sign for the multi-sig account
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

    // Set up the multi-sig wallets that are authorized to sign for the multi-sig user
    // Each wallet must belong to a user that has been added as an authorized signer
    let multi_sig_wallets = setup_multi_sig_wallets();

    // The outer signer is required to be an authorized user or an agent of the
    // authorized user of the multi-sig user.

    // Address of the multi-sig user that the action will be executed for
    // Executing the action requires at least the specified threshold of signatures
    // required for that multi-sig user
    let multi_sig_user: Address = "0x0000000000000000000000000000000000000005"
        .parse()
        .unwrap();

    info!("=== Multi-Sig Order Example ===");
    info!("Multi-sig user address: {}", multi_sig_user);
    info!("Outer signer (current wallet): {}", address);
    info!(
        "Exchange client connected to: {:?}",
        exchange_client.http_client.base_url
    );
    info!(
        "Authorized wallets ({} total): {:?}",
        multi_sig_wallets.len(),
        multi_sig_wallets
            .iter()
            .map(|w| w.address())
            .collect::<Vec<_>>()
    );

    // Define the multi-sig inner action - in this case, placing an order
    // This matches the Python example: asset index 4, buy, price 1100, size 0.2
    let order = ClientOrderRequest {
        asset: "ETH".to_string(), // Asset index 4 in Python corresponds to ETH
        is_buy: true,
        reduce_only: false,
        limit_px: 1100.0,
        sz: 0.2,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    info!("");
    info!("Order details: {:?}", order);
    info!("Executing multi-sig order...");
    info!(
        "Collecting signatures from {} authorized wallets...",
        multi_sig_wallets.len()
    );

    // Execute the multi-sig order
    // This will collect signatures from all provided wallets and submit them together
    // The action will only succeed if enough valid signatures are provided (>= threshold)
    match exchange_client
        .multi_sig_order(multi_sig_user, order, &multi_sig_wallets)
        .await
    {
        Ok(response) => {
            info!("✓ Multi-sig order placed successfully!");
            info!("Response: {:?}", response);
        }
        Err(e) => {
            info!("✗ Multi-sig order failed: {}", e);
            info!("");
            info!("This is expected if:");
            info!("  • The multi-sig user is not properly configured");
            info!("  • The provided wallets are not authorized signers");
            info!("  • Not enough signatures provided to meet threshold");
            info!("");
            info!("To use in production:");
            info!("  1. Convert a user to multi-sig: convert_to_multi_sig()");
            info!("  2. Add authorized addresses: update_multi_sig_addresses()");
            info!("  3. Use those authorized wallets to sign transactions");
            info!("  4. Ensure you provide >= threshold number of valid signatures");
        }
    }

    info!("");
    info!("Example completed");
}
