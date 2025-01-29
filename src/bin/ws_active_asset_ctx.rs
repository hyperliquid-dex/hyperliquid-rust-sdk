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
    let coin = "BTC".to_string();

    let (sender, mut receiver) = unbounded_channel();
    let subscription_id = info_client
        .subscribe(Subscription::ActiveAssetCtx { coin }, sender)
        .await
        .unwrap();

    spawn(async move {
        sleep(Duration::from_secs(30)).await;
        info!("Unsubscribing from active asset ctx");
        info_client.unsubscribe(subscription_id).await.unwrap()
    });

    while let Some(Message::ActiveAssetCtx(active_asset_ctx)) = receiver.recv().await {
        info!("Received active asset ctx: {active_asset_ctx:?}");
    }
}
