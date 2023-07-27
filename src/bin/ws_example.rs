use log::debug;

use hyperliquid_rust_sdk::{InfoClient, Subscription};
use tokio::{
    spawn,
    sync::mpsc::unbounded_channel,
    time::{sleep, Duration},
};

#[tokio::main]
async fn main() {
    let mut info_client = InfoClient::new(None, Some("https://api.hyperliquid-testnet.xyz"))
        .await
        .unwrap();

    let (sender, mut receiver) = unbounded_channel();
    let subscription_id = info_client
        .subscribe(
            Subscription::Trades {
                coin: "ETH".to_string(),
            },
            sender,
        )
        .await
        .unwrap();

    spawn(async move {
        sleep(Duration::from_secs(30)).await;
        debug!("Unsubscribing");
        info_client.unsubscribe(subscription_id).await.unwrap()
    });

    loop {
        let ret = receiver.recv().await.unwrap_or_default();
        if ret.is_empty() {
            // we've unsubscribed
            break;
        }
        println!("received data: {ret}")
    }
}
