use env_logger::{Builder, Target};
use hyperliquid_rust_sdk::{InfoClient, SubscriptionType};
use tokio::{
    spawn,
    sync::mpsc::unbounded_channel,
    time::{sleep, Duration},
};

#[tokio::main]
async fn main() {
    Builder::new().target(Target::Stdout).init();

    let mut exchange_client =
        InfoClient::new(None, Some("https://api.hyperliquid-testnet.xyz"), false)
            .await
            .unwrap();

    let (sender, mut receiver) = unbounded_channel();
    exchange_client
        .subscribe(
            SubscriptionType::Trades {
                coin: "SOL".to_string(),
            },
            sender,
        )
        .await
        .unwrap();

    let (sender2, mut receiver2) = unbounded_channel();
    exchange_client
        .subscribe(
            SubscriptionType::Trades {
                coin: "ETH".to_string(),
            },
            sender2,
        )
        .await
        .unwrap();

    // let sub_id1 = sub_id;

    spawn(async move {
        sleep(Duration::from_secs(60)).await;
        println!("UNSUBSCRIBING");
        exchange_client.unsubscribe(1).await.unwrap()
    });

    spawn(async move {
        loop {
            let ret: String = receiver2.recv().await.unwrap_or_default();
            if ret.is_empty() {
                break;
            }
            println!("subscription 1: {ret}")
        }
    });

    loop {
        let ret = receiver.recv().await.unwrap_or_default();
        println!("subscription 0: {ret}")
    }
}
