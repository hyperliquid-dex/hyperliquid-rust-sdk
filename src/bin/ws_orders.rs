use alloy::primitives::{Address, U256};

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

    let address = "0x1234567890123456789012345678901234567890"
        .parse::<Address>()
        .unwrap();
    let mut info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();

    let (sender, mut receiver) = unbounded_channel();
    let subscription_id = info_client
        .subscribe(Subscription::UserEvents { user: address }, sender)
        .await
        .unwrap();

    spawn(async move {
        sleep(Duration::from_secs(30)).await;
        info!("Unsubscribing from user events");
        info_client.unsubscribe(subscription_id).await.unwrap()
    });

    // this loop ends when we unsubscribe
    while let Some(Message::User(user_events)) = receiver.recv().await {
        info!("Received user events: {:?}", user_events);
    }
}
