use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};
use log::info;

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

    // The multi-sig user address (this would be the address that was converted to multi-sig)
    let multi_sig_user: Address = "0x0000000000000000000000000000000000000005"
        .parse()
        .unwrap();

    // Set up the multi-sig wallets that are authorized to sign for the multi-sig user
    let multi_sig_wallets = setup_multi_sig_wallets();

    info!("Multi-sig user: {}", multi_sig_user);
    info!("Outer signer (current wallet): {}", address);
    info!(
        "Exchange client connected to: {:?}",
        exchange_client.http_client.base_url
    );
    info!(
        "Multi-sig wallets: {:?}",
        multi_sig_wallets
            .iter()
            .map(|w| w.address())
            .collect::<Vec<_>>()
    );

    info!("Multi-sig register token functionality requires custom action handling");
    info!("The spot token registration action (spotDeploy) is a complex action that:");
    info!("1. Requires specific permission levels");
    info!("2. Has custom parameters for token specification");
    info!("3. Is typically used for specialized spot market operations");
    info!("");
    info!("For multi-sig spot token registration, you would:");
    info!("1. Create a custom spotDeploy action with registerToken2 parameters");
    info!("2. Hash the action using the Actions::hash method");
    info!("3. Sign with multiple authorized wallets using sign_l1_action_multi_sig");
    info!("4. Submit using the post_multi_sig method");
    info!("");
    info!("This is an advanced operation - consult Hyperliquid documentation for details");

    info!("Example completed - multi-sig register token requirements explained");
}
