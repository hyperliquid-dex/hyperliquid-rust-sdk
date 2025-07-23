use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1)
        .expect("Usage: user_fills_by_time <address> [start_hours_ago] [end_hours_ago]");

    let user_address: H160 = address.parse()?;

    // Default to last 24 hours if no time range specified
    let start_hours_ago: u64 = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "24".to_string())
        .parse()
        .expect("Invalid start_hours_ago");

    let end_hours_ago: Option<u64> = std::env::args()
        .nth(3)
        .map(|s| s.parse().expect("Invalid end_hours_ago"));

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let start_time = current_time - (start_hours_ago * 60 * 60 * 1000);
    let end_time = end_hours_ago.map(|hours| current_time - (hours * 60 * 60 * 1000));

    let info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;

    match info_client.user_fills_by_time(user_address, start_time, end_time).await {
        Ok(fills) => {
            println!("User fills by time for {} (last {} hours): {:#?}", 
                     address, start_hours_ago, fills);
            println!("Total fills: {}", fills.len());
        }
        Err(e) => {
            eprintln!("Error getting user fills by time: {}", e);
        }
    }

    Ok(())
} 