use hyperliquid_rust_sdk::{BaseUrl, InfoClient};

#[tokio::main]
async fn main() {
    let info_client = InfoClient::new(None, Some(BaseUrl::Testnet))
        .await
        .expect("Failed to create info client");

    match info_client.query_perp_deploy_auction_status().await {
        Ok(result) => {
            println!("Perp deploy auction status:");
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Err(e) => {
            eprintln!("Error querying perp deploy auction status: {e:?}");
        }
    }
}