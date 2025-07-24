use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1)
        .expect("Usage: frontend_open_orders <address>");

    let user_address: H160 = address.parse()?;

    let info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;

    match info_client.frontend_open_orders(user_address).await {
        Ok(frontend_orders) => {
            println!("Frontend open orders for {}: {:#?}", address, frontend_orders);
        }
        Err(e) => {
            eprintln!("Error getting frontend open orders: {}", e);
        }
    }

    Ok(())
} 