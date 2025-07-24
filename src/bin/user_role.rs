use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1)
        .expect("Usage: user_role <address>");

    let user_address: H160 = address.parse()?;

    let info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await?;

    match info_client.user_role(user_address).await {
        Ok(role_info) => {
            println!("User role for {}: {:#?}", address, role_info);
            match role_info.role.as_str() {
                "user" => println!("Account type: Regular user"),
                "agent" => println!("Account type: API agent"),
                "vault" => println!("Account type: Vault"),
                "subAccount" => println!("Account type: Sub-account"),
                "missing" => println!("Account type: Missing/Unknown"),
                _ => println!("Account type: Unknown ({})", role_info.role),
            }
        }
        Err(e) => {
            eprintln!("Error getting user role: {}", e);
        }
    }

    Ok(())
} 