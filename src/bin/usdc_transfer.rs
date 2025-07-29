use alloy::signers::local::PrivateKeySigner;
use hyperliquid_rust_sdk::{BaseUrl, ExchangeClient, ExchangeResponseStatus};
use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let wallet: PrivateKeySigner =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    let amount = "1";
    let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";

    match exchange_client.usdc_transfer(amount, destination, None).await {
        Ok(status) => {
            match status {
                ExchangeResponseStatus::Ok(data) => {
                    
                    info!("Transfer successful: {data:?}");
                }
                ExchangeResponseStatus::Err(err) => {

                    eprintln!("Exchange error: {err:?}");
                }
            }
        }
        Err(hyperliquid_err) => {

            eprintln!("Technical error: {hyperliquid_err:?}");
        }
    }
}
