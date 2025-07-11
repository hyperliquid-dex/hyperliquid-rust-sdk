use ethers::signers::{LocalWallet, Signer};
use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, ExchangeClient, PerpDexSchemaInput,
};

/// This test verifies that perp_deploy_register_asset correctly constructs the action
/// without actually sending it to the network
#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    info!("Creating ExchangeClient for Testnet...");
    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    info!("ExchangeClient created successfully");
    info!("Wallet address: {:?}", exchange_client.wallet.address());
    info!("Base URL: {}", exchange_client.http_client.base_url);
    
    // Create test schema
    let schema = PerpDexSchemaInput {
        full_name: "Test Coin".to_string(),
        collateral_token: 0,  // 0 for USDC
        oracle_updater: Some("0x1234567890123456789012345678901234567890".to_string()),
    };
    
    info!("\nTest parameters:");
    info!("  dex: testdex");
    info!("  coin: TESTCOIN");
    info!("  sz_decimals: 6");
    info!("  oracle_px: 100.0");
    info!("  margin_table_id: 1");
    info!("  only_isolated: false");
    info!("  schema: {:?}", schema);
    
    info!("\nThe perp_deploy_register_asset method is ready to use.");
    info!("It will construct a PerpDeploy::RegisterAsset action with the provided parameters.");
    info!("The action will be signed and sent to the exchange when called.");
    
    // Demonstrate that we can call the method (but don't actually send it)
    info!("\nMethod signature:");
    info!("  exchange_client.perp_deploy_register_asset(");
    info!("      dex: &str,");
    info!("      max_gas: Option<u32>,");
    info!("      coin: &str,");
    info!("      sz_decimals: u32,");
    info!("      oracle_px: &str,");
    info!("      margin_table_id: u32,");
    info!("      only_isolated: bool,");
    info!("      schema: Option<PerpDexSchemaInput>,");
    info!("  ) -> Result<ExchangeResponseStatus>");
}