use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 8 {
        eprintln!("Usage: twap_order <private_key> <asset> <is_buy> <size> <minutes> <reduce_only> <randomize>");
        eprintln!("Example: twap_order 0x1234... ETH true 1.0 30 false true");
        std::process::exit(1);
    }

    let private_key = &args[1];
    let asset = &args[2];
    let is_buy = args[3].parse::<bool>()?;
    let size = args[4].parse::<f64>()?;
    let minutes = args[5].parse::<u32>()?;
    let reduce_only = args[6].parse::<bool>()?;
    let randomize = args[7].parse::<bool>()?;

    let wallet: LocalWallet = private_key.parse()?;
    
    let exchange_client = ExchangeClient::new(
        None,
        wallet,
        Some(BaseUrl::Testnet),
        None,
        None,
    ).await?;

    println!("Placing TWAP order:");
    println!("  Asset: {}", asset);
    println!("  Side: {}", if is_buy { "BUY" } else { "SELL" });
    println!("  Size: {}", size);
    println!("  Duration: {} minutes", minutes);
    println!("  Reduce Only: {}", reduce_only);
    println!("  Randomize: {}", randomize);

    match exchange_client.twap_order(
        asset,
        is_buy,
        size,
        reduce_only,
        minutes,
        randomize,
        None
    ).await {
        Ok(response) => {
            println!("✅ TWAP order placed successfully!");
            println!("Response: {:#?}", response);
        }
        Err(e) => {
            eprintln!("❌ Error placing TWAP order: {}", e);
        }
    }

    Ok(())
} 