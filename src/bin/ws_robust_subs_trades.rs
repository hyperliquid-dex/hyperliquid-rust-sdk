use hyperliquid_rust_sdk::Message;
use hyperliquid_rust_sdk::{robust::Subs, BaseUrl, Subscription};
use tokio::{
    sync::mpsc,
    time::{sleep, Duration},
};

/// Stream trades for BTC/USD using the subscription helper
#[tokio::main]
async fn main() {
    env_logger::init();

    let (subs, handle) = Subs::start(&BaseUrl::Mainnet);

    let (sub_tx, mut sub_rx) = mpsc::unbounded_channel();

    tokio::select! {
        join_result = handle => join_result.unwrap().unwrap(),
        _ = async {
            while let Some(message) = sub_rx.recv().await {
                if let Message::Trades(trades) = message {
                    for trade in trades.data {
                        println!("{} {}", trade.side, trade.px);
                    }
                }
            }
        } => {},
        _ = async {
                let _sub_token = subs
                    .add(
                        Subscription::Trades {
                            coin: "BTC".to_string(),
                        },
                        sub_tx,
                    )
                    .await
                    .unwrap();

                println!("Streaming BTC/USD trades for 60 sec...");

                sleep(Duration::from_secs(60)).await;

                subs.cancel().await;
        } => {}
    };

    // The sub token was dropped here, causing an unsubscribe
    println!("Finished streaming. Unsubscribing and exiting in 1 sec...");

    sleep(Duration::from_secs(1)).await;
}
