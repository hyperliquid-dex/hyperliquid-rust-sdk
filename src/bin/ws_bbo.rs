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
        .subscribe(Subscription::Bbo { coin }, sender)
        .await
        .unwrap();

    spawn(async move {
        sleep(Duration::from_secs(30)).await;
        info!("Unsubscribing from bbo");
        info_client.unsubscribe(subscription_id).await.unwrap()
    });

    while let Some(Message::Bbo(bbo)) = receiver.recv().await {
        info!("Received bbo: {bbo:?}");
    }
}
