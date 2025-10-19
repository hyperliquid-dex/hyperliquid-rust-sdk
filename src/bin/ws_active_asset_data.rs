use alloy::primitives::address;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use log::info;
use tokio::{
    spawn,
    sync::mpsc::unbounded_channel,
    time::{sleep, Duration},
};

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();
    let user = address!("0xc64cc00b46101bd40aa1c3121195e85c0b0918d8");
    let coin = "BTC".to_string();

    let (sender, mut receiver) = unbounded_channel();
    let subscription_id = info_client
        .subscribe(Subscription::ActiveAssetData { user, coin }, sender)
        .await
        .unwrap();

    spawn(async move {
        sleep(Duration::from_secs(30)).await;
        info!("Unsubscribing from active asset data");
        info_client.unsubscribe(subscription_id).await.unwrap()
    });

    while let Some(Message::ActiveAssetData(active_asset_data)) = receiver.recv().await {
        info!("Received active asset data: {active_asset_data:?}");
    }
}
