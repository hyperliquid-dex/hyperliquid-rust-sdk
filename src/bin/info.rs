use ethers::types::H160;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};
use log::info;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    env_logger::init();
    let info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();
    open_orders_example(&info_client).await;
    user_state_example(&info_client).await;
    user_states_example(&info_client).await;
    recent_trades(&info_client).await;
    meta_example(&info_client).await;
    meta_and_asset_contexts_example(&info_client).await;
    all_mids_example(&info_client).await;
    user_fills_example(&info_client).await;
    funding_history_example(&info_client).await;
    l2_snapshot_example(&info_client).await;
    candles_snapshot_example(&info_client).await;
}

async fn open_orders_example(info_client: &InfoClient) {
    let user: String = "0xc64cc00b46101bd40aa1c3121195e85c0b0918d8"
        .parse()
        .unwrap();
    let user = H160::from_str(&user).unwrap();

    info!(
        "Open order data for {user}: {:?}",
        info_client.open_orders(user).await.unwrap()
    );
}

async fn user_state_example(info_client: &InfoClient) {
    let user: String = "0xc64cc00b46101bd40aa1c3121195e85c0b0918d8"
        .parse()
        .unwrap();
    let user = H160::from_str(&user).unwrap();

    info!(
        "User state data for {user}: {:?}",
        info_client.user_state(user).await.unwrap()
    );
}

async fn user_states_example(info_client: &InfoClient) {
    let user: String = "0xc64cc00b46101bd40aa1c3121195e85c0b0918d8"
        .parse()
        .unwrap();
    let user = H160::from_str(&user).unwrap();

    info!(
        "User state data for {user}: {:?}",
        info_client.user_states(vec![user]).await.unwrap()
    );
}

async fn recent_trades(info_client: &InfoClient) {
    let coin = "ETH";

    info!(
        "Recent trades for {coin}: {:?}",
        info_client.recent_trades(coin.to_string()).await.unwrap()
    );
}

async fn meta_example(info_client: &InfoClient) {
    info!("Metadata: {:?}", info_client.meta().await.unwrap());
}

async fn meta_and_asset_contexts_example(info_client: &InfoClient) {
    info!(
        "MetaAndAssetContexts: {:?}",
        info_client.meta_and_asset_contexts().await.unwrap()
    );
}

async fn all_mids_example(info_client: &InfoClient) {
    info!("All mids: {:?}", info_client.all_mids().await.unwrap());
}

async fn user_fills_example(info_client: &InfoClient) {
    let user: String = "0xc64cc00b46101bd40aa1c3121195e85c0b0918d8"
        .parse()
        .unwrap();
    let user = H160::from_str(&user).unwrap();

    info!(
        "User fills data for {user}: {:?}",
        info_client.user_fills(user).await.unwrap()
    );
}

async fn funding_history_example(info_client: &InfoClient) {
    let coin = "ETH";

    let start_timestamp = 1690540602225;
    let end_timestamp = 1690569402225;
    info!(
        "Funding data history for {coin} between timestamps {start_timestamp} and {end_timestamp}: {:?}",
        info_client.funding_history(coin.to_string(), start_timestamp, Some(end_timestamp)).await.unwrap()
    );
}

async fn l2_snapshot_example(info_client: &InfoClient) {
    let coin = "ETH";

    info!(
        "L2 snapshot data for {coin}: {:?}",
        info_client.l2_snapshot(coin.to_string()).await.unwrap()
    );
}

async fn candles_snapshot_example(info_client: &InfoClient) {
    let coin = "ETH";
    let start_timestamp = 1690540602225;
    let end_timestamp = 1690569402225;
    let interval = "1h";

    info!(
        "Candles snapshot data for {coin} between timestamps {start_timestamp} and {end_timestamp} with interval {interval}: {:?}",
        info_client
            .candles_snapshot(coin.to_string(), interval.to_string(), start_timestamp, end_timestamp)
            .await
            .unwrap()
    );
}
