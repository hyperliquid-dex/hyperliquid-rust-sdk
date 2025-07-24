use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: twap_cancel <private_key> <asset> <twap_id>");
        eprintln!("Example: twap_cancel 0x1234... ETH 12345");
        std::process::exit(1);
    }

    let private_key = &args[1];
    let asset = &args[2];
    let twap_id = args[3].parse::<u32>()?;

    let wallet: LocalWallet = private_key.parse()?;
    
    let exchange_client = ExchangeClient::new(
        None,
        wallet,
        Some(BaseUrl::Testnet),
        None,
        None,
    ).await?;

    println!("Cancelling TWAP order:");
    println!("  Asset: {}", asset);
    println!("  TWAP ID: {}", twap_id);

    match exchange_client.twap_cancel(asset, twap_id, None).await {
        Ok(response) => {
            println!("✅ TWAP order cancelled successfully!");
            println!("Response: {:#?}", response);
        }
        Err(e) => {
            eprintln!("❌ Error cancelling TWAP order: {}", e);
        }
    }

    Ok(())
} 