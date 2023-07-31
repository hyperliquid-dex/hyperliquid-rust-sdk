use log::info;

use ethers::signers::{LocalWallet, Signer};
use hyperliquid_rust_sdk::{
    ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient, TESTNET_API_URL,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(TESTNET_API_URL), None, None)
        .await
        .unwrap();

    /*
        Create a new wallet with the agent.
        This agent cannot transfer or withdraw funds, but can for example place orders.
    */

    let (private_key, response) = exchange_client.approve_agent().await.unwrap();
    info!("Agent creation response: {response:?}");

    let wallet: LocalWallet = private_key.parse().unwrap();

    info!("Agent address: {:?}", wallet.address());

    let exchange_client = ExchangeClient::new(None, wallet, Some(TESTNET_API_URL), None, None)
        .await
        .unwrap();

    let order = ClientOrderRequest {
        asset: "ETH".to_string(),
        is_buy: true,
        reduce_only: false,
        limit_px: 1795.0,
        sz: 0.01,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let response = exchange_client.order(order).await.unwrap();

    info!("Order placed: {response:?}");
}
