use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1)
        .expect("Usage: portfolio <address>");

    let user_address: H160 = address.parse()?;

    let info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;

    match info_client.portfolio(user_address).await {
        Ok(portfolio) => {
            println!("Portfolio for {}: {:#?}", address, portfolio);
            
            for period_data in &portfolio {
                println!("\n=== {} Period ===", period_data.period);
                println!("Volume: {}", period_data.data.vlm);
                println!("Account value history points: {}", period_data.data.account_value_history.len());
                println!("PnL history points: {}", period_data.data.pnl_history.len());
                
                if !period_data.data.account_value_history.is_empty() {
                    let latest = &period_data.data.account_value_history[period_data.data.account_value_history.len() - 1];
                    println!("Latest account value: {} (timestamp: {})", latest.1, latest.0);
                }
                
                if !period_data.data.pnl_history.is_empty() {
                    let latest_pnl = &period_data.data.pnl_history[period_data.data.pnl_history.len() - 1];
                    println!("Latest PnL: {} (timestamp: {})", latest_pnl.1, latest_pnl.0);
                }
            }
        }
        Err(e) => {
            eprintln!("Error getting portfolio: {}", e);
        }
    }

    Ok(())
} 