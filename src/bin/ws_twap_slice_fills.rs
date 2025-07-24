use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use log::info;
use tokio::{
    spawn,
    sync::mpsc::unbounded_channel,
    time::{sleep, Duration},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ws_twap_slice_fills <address>");
        eprintln!("Example: ws_twap_slice_fills 0x1234567890123456789012345678901234567890");
        std::process::exit(1);
    }

    let address_str = &args[1];
    let user_address: H160 = address_str.parse()?;

    let mut info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await?;

    let (sender, mut receiver) = unbounded_channel();
    let subscription_id = info_client
        .subscribe(Subscription::UserTwapSliceFills { user: user_address }, sender)
        .await?;

    info!("📡 Subscribed to TWAP slice fills for address: {}", address_str);
    info!("Monitoring for real-time TWAP slice fill updates...");

    // Unsubscribe after 60 seconds for demo purposes
    spawn(async move {
        sleep(Duration::from_secs(60)).await;
        info!("🔚 Unsubscribing from TWAP slice fills (60 second demo)");
        info_client.unsubscribe(subscription_id).await.unwrap();
    });

    // Listen for TWAP slice fill updates
    while let Some(message) = receiver.recv().await {
        match message {
            Message::UserTwapSliceFills(twap_fills) => {
                info!("🔥 Received TWAP slice fills update!");
                
                let data = &twap_fills.data;
                info!("User: {}", data.user);
                
                if let Some(is_snapshot) = data.is_snapshot {
                    if is_snapshot {
                        info!("📸 This is a snapshot with {} historical fills", data.twap_slice_fills.len());
                    } else {
                        info!("🆕 This is a real-time update with {} new fills", data.twap_slice_fills.len());
                    }
                }

                for (i, twap_fill) in data.twap_slice_fills.iter().enumerate() {
                    info!("  TWAP Fill #{}", i + 1);
                    info!("    TWAP ID: {}", twap_fill.twap_id);
                    info!("    Coin: {}", twap_fill.fill.coin);
                    info!("    Side: {}", twap_fill.fill.side);
                    info!("    Price: {}", twap_fill.fill.px);
                    info!("    Size: {}", twap_fill.fill.sz);
                    info!("    Direction: {}", twap_fill.fill.dir);
                    info!("    Fee: {} {}", twap_fill.fill.fee, twap_fill.fill.fee_token);
                    info!("    Time: {}", twap_fill.fill.time);
                    info!("    Order ID: {}", twap_fill.fill.oid);
                }
            }
            _ => {
                // Handle other message types if needed
                info!("📨 Received other message type");
            }
        }
    }

    info!("🏁 TWAP slice fills monitoring completed");
    Ok(())
} 