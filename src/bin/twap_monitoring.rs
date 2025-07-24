use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: twap_monitoring <address>");
        eprintln!("Example: twap_monitoring 0x1234567890123456789012345678901234567890");
        std::process::exit(1);
    }

    let address_str = &args[1];
    let address: H160 = address_str.parse()?;

    let info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await?;

    println!("Monitoring TWAP slice fills for address: {}", address_str);
    println!("{}", "=".repeat(60));

    match info_client.user_twap_slice_fills(address).await {
        Ok(twap_fills) => {
            if twap_fills.is_empty() {
                println!("No TWAP slice fills found for this address.");
            } else {
                println!("Found {} TWAP slice fills:", twap_fills.len());
                println!();

                for (i, twap_fill) in twap_fills.iter().enumerate() {
                    println!("TWAP Fill #{}", i + 1);
                    println!("  TWAP ID: {}", twap_fill.twap_id);
                    println!("  Coin: {}", twap_fill.fill.coin);
                    println!("  Side: {}", twap_fill.fill.side);
                    println!("  Price: {}", twap_fill.fill.px);
                    println!("  Size: {}", twap_fill.fill.sz);
                    println!("  Direction: {}", twap_fill.fill.dir);
                    println!("  Fee: {} {}", twap_fill.fill.fee, twap_fill.fill.fee_token);
                    println!("  Crossed: {}", twap_fill.fill.crossed);
                    println!("  Closed PnL: {}", twap_fill.fill.closed_pnl);
                    println!("  Time: {}", twap_fill.fill.time);
                    println!("  Order ID: {}", twap_fill.fill.oid);
                    println!("  Trade ID: {}", twap_fill.fill.tid);
                    println!("  Hash: {}", twap_fill.fill.hash);
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error fetching TWAP slice fills: {}", e);
        }
    }

    Ok(())
} 