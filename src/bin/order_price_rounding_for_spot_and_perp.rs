use ethers::signers::LocalWallet;
use log::info;

use hyperliquid_rust_sdk::{
    BaseUrl, ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest, ExchangeClient,
    ExchangeDataStatus, ExchangeResponseStatus,
};
use std::{thread::sleep, time::Duration};

#[tokio::main]
async fn main() {
    env_logger::init();
    // Key was randomly generated for testing and shouldn't be used with any real funds
    let wallet: LocalWallet = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse()
        .unwrap();

    let exchange_client = ExchangeClient::new(None, wallet, Some(BaseUrl::Testnet), None, None)
        .await
        .unwrap();

    let order = ClientOrderRequest {
        asset: "PURR/USDC".to_string(),
        is_buy: true,
        reduce_only: false,
        // Replace this with round_perp() when making a order of a perp asset
        limit_px: round_spot(78.2323232322332, 0), // This will be rounded to 78.232
        sz: 10.0,
        cloid: None,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Gtc".to_string(),
        }),
    };

    let response = exchange_client.order(order, None).await.unwrap();
    info!("Order placed: {response:?}");

    let response = match response {
        ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
        ExchangeResponseStatus::Err(e) => panic!("error with exchange response: {e}"),
    };
    let status = response.data.unwrap().statuses[0].clone();
    let oid = match status {
        ExchangeDataStatus::Filled(order) => order.oid,
        ExchangeDataStatus::Resting(order) => order.oid,
        _ => panic!("Error: {status:?}"),
    };

    // So you can see the order before it's cancelled
    sleep(Duration::from_secs(10));

    let cancel = ClientCancelRequest {
        asset: "PURR/USDC".to_string(),
        oid,
    };

    // This response will return an error if order was filled (since you can't cancel a filled order), otherwise it will cancel the order
    let response = exchange_client.cancel(cancel, None).await.unwrap();
    info!("Order potentially cancelled: {response:?}");
}

/// .
///
/// # Rounding Spot Price
///
/// For spot price, when placing the order, the price should contain 5 significant digits and at
/// most 8 decimal places. Refer to: https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/tick-and-lot-size
fn round_spot(num: f64, sz_decimals: u16) -> f64 {
    round_price(num, 5, (8 - sz_decimals) as i32)
}

/// 
/// # Rounding Perp Price
///
/// For perp price, when placing the order, the price should contain 5 significant digits and at
/// most 6 decimal places. Refer to: https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/tick-and-lot-size
fn round_perp(num: f64, sz_decimals: u16) -> f64 {
    round_price(num, 5, (6 - sz_decimals) as i32)
}

/// .
///
/// # Rounding Price
///
/// Rounding price common function
fn round_price(num: f64, significant_digits: i32, max_decimal_places: i32) -> f64 {
    if num == 0.0 {
        return 0.0_f64; // Handle zero separately
    }

    let order_of_magnitude = num.abs().log10().floor() as i32;

    // Calculate needed decimal places to maintain 5 significant digits
    let needed_decimal_places = significant_digits - order_of_magnitude - 1;

    // Determine actual decimal places, considering the maximum limit
    let actual_decimal_places = if needed_decimal_places > max_decimal_places {
        max_decimal_places
    } else if needed_decimal_places < 0 {
        0
    } else {
        needed_decimal_places
    };

    // Format the number with the appropriate number of decimal places
    let value = format!("{:.*}", actual_decimal_places as usize, num)
        .parse::<f64>()
        .unwrap();

    value
}
