/*
This is an example of a basic market making strategy.

We subscribe to the current mid price and build a market around this price. Whenever our market becomes outdated, we place and cancel orders to renew it.
*/
use alloy::signers::local::PrivateKeySigner;

use hyperliquid_rust_sdk::{MarketMaker, MarketMakerInput};

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: PrivateKeySigner =
        "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse()
            .unwrap();
    let market_maker_input = MarketMakerInput {
        asset: "ETH".to_string(),
        target_liquidity: 0.25,
        max_bps_diff: 2,
        half_spread: 1,
        max_absolute_position_size: 0.5,
        decimals: 1,
        wallet,
    };
    MarketMaker::new(market_maker_input).await.start().await
}
