use hyperliquid_rust_sdk::{robust::Stream, BaseUrl, Message, Subscription, SubscriptionSendData};
use std::time::Duration;
use tokio::{spawn, sync::mpsc, time::sleep};

/// Stream 1m ETH/USD candles directly without using any subscription helper
#[tokio::main]
async fn main() {
    env_logger::init();

    let (inbox_tx, mut inbox_rx) = mpsc::channel(100);

    let (stream, handle) = Stream::connect(&BaseUrl::Mainnet, inbox_tx);

    stream
        .send(
            serde_json::to_value(SubscriptionSendData {
                method: "subscribe",
                subscription: &serde_json::to_value(Subscription::Candle {
                    coin: "ETH".to_string(),
                    interval: "1m".to_string(),
                })
                .unwrap(),
            })
            .unwrap(),
        )
        .await
        .unwrap();

    println!("Streaming ETH/USD 1m candles for 60 seconds...");
    println!("volume\topen\thigh\tlow\tclose");

    spawn(async move {
        sleep(Duration::from_secs(60)).await;

        stream.cancel().await;
    });

    while let Some(message) = inbox_rx.recv().await {
        if let Message::Candle(candle) = message {
            let data = candle.data;
            // vol, open, high, low, close
            println!(
                "{}\t{}\t{}\t{}\t{}",
                data.volume, data.open, data.high, data.low, data.close
            );
        }
    }

    handle.await.unwrap().unwrap();
}
