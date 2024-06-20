use ethers::{
    signers::{LocalWallet, Signer},
    types::H160,
};
use log::{error, info};

use tokio::sync::mpsc::unbounded_channel;

use crate::{
    bps_diff, truncate_float, BaseUrl, ClientCancelRequest, ClientLimit, ClientOrder,
    ClientOrderRequest, ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus, InfoClient,
    Message, Subscription, UserData, EPSILON,
};
#[derive(Debug)]
pub struct MarketMakerRestingOrder {
    pub oid: u64,
    pub position: f64,
    pub price: f64,
}

pub struct MarketMakerInput {
    pub asset: String,
    pub target_liquidity: f64, // Amount of liquidity on both sides to target
    pub half_spread: u16,      // Half of the spread for our market making (in BPS)
    pub max_bps_diff: u16, // Max deviation before we cancel and put new orders on the book (in BPS)
    pub max_absolute_position_size: f64, // Absolute value of the max position we can take on
    pub decimals: u32,     // Decimals to round to for pricing
    pub wallet: LocalWallet, // Wallet containing private key
}

pub struct MarketMaker {
    pub asset: String,
    pub target_liquidity: f64,
    pub half_spread: u16,
    pub max_bps_diff: u16,
    pub max_absolute_position_size: f64,
    pub decimals: u32,
    pub lower_resting: MarketMakerRestingOrder,
    pub upper_resting: MarketMakerRestingOrder,
    pub cur_position: f64,
    pub latest_mid_price: f64,
    pub info_client: InfoClient,
    pub exchange_client: ExchangeClient,
    pub user_address: H160,
}

impl MarketMaker {
    pub async fn new(input: MarketMakerInput) -> MarketMaker {
        let user_address = input.wallet.address();

        let info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();
        let exchange_client =
            ExchangeClient::new(None, input.wallet, Some(BaseUrl::Testnet), None, None)
                .await
                .unwrap();

        MarketMaker {
            asset: input.asset,
            target_liquidity: input.target_liquidity,
            half_spread: input.half_spread,
            max_bps_diff: input.max_bps_diff,
            max_absolute_position_size: input.max_absolute_position_size,
            decimals: input.decimals,
            lower_resting: MarketMakerRestingOrder {
                oid: 0,
                position: 0.0,
                price: -1.0,
            },
            upper_resting: MarketMakerRestingOrder {
                oid: 0,
                position: 0.0,
                price: -1.0,
            },
            cur_position: 0.0,
            latest_mid_price: -1.0,
            info_client,
            exchange_client,
            user_address,
        }
    }

    pub async fn start(&mut self) {
        let (sender, mut receiver) = unbounded_channel();

        // Subscribe to UserEvents for fills
        self.info_client
            .subscribe(
                Subscription::UserEvents {
                    user: self.user_address,
                },
                sender.clone(),
            )
            .await
            .unwrap();

        // Subscribe to AllMids so we can market make around the mid price
        self.info_client
            .subscribe(Subscription::AllMids, sender)
            .await
            .unwrap();

        loop {
            let message = receiver.recv().await.unwrap();
            match message {
                Message::AllMids(all_mids) => {
                    let all_mids = all_mids.data.mids;
                    let mid = all_mids.get(&self.asset);
                    if let Some(mid) = mid {
                        let mid: f64 = mid.parse().unwrap();
                        self.latest_mid_price = mid;
                        // Check to see if we need to cancel or place any new orders
                        self.potentially_update().await;
                    } else {
                        error!(
                            "could not get mid for asset {}: {all_mids:?}",
                            self.asset.clone()
                        );
                    }
                }
                Message::User(user_events) => {
                    // We haven't seen the first mid price event yet, so just continue
                    if self.latest_mid_price < 0.0 {
                        continue;
                    }
                    let user_events = user_events.data;
                    if let UserData::Fills(fills) = user_events {
                        for fill in fills {
                            let amount: f64 = fill.sz.parse().unwrap();
                            // Update our resting positions whenever we see a fill
                            if fill.side.eq("B") {
                                self.cur_position += amount;
                                self.lower_resting.position -= amount;
                                info!("Fill: bought {amount} {}", self.asset.clone());
                            } else {
                                self.cur_position -= amount;
                                self.upper_resting.position -= amount;
                                info!("Fill: sold {amount} {}", self.asset.clone());
                            }
                        }
                    }
                    // Check to see if we need to cancel or place any new orders
                    self.potentially_update().await;
                }
                _ => {
                    panic!("Unsupported message type");
                }
            }
        }
    }

    async fn attempt_cancel(&self, asset: String, oid: u64) -> bool {
        let cancel = self
            .exchange_client
            .cancel(ClientCancelRequest { asset, oid }, None)
            .await;

        match cancel {
            Ok(cancel) => match cancel {
                ExchangeResponseStatus::Ok(cancel) => {
                    if let Some(cancel) = cancel.data {
                        if !cancel.statuses.is_empty() {
                            match cancel.statuses[0].clone() {
                                ExchangeDataStatus::Success => {
                                    return true;
                                }
                                ExchangeDataStatus::Error(e) => {
                                    error!("Error with cancelling: {e}")
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            error!("Exchange data statuses is empty when cancelling: {cancel:?}")
                        }
                    } else {
                        error!("Exchange response data is empty when cancelling: {cancel:?}")
                    }
                }
                ExchangeResponseStatus::Err(e) => error!("Error with cancelling: {e}"),
            },
            Err(e) => error!("Error with cancelling: {e}"),
        }
        false
    }

    async fn place_order(
        &self,
        asset: String,
        amount: f64,
        price: f64,
        is_buy: bool,
    ) -> (f64, u64) {
        let order = self
            .exchange_client
            .order(
                ClientOrderRequest {
                    asset,
                    is_buy,
                    reduce_only: false,
                    limit_px: price,
                    sz: amount,
                    cloid: None,
                    order_type: ClientOrder::Limit(ClientLimit {
                        tif: "Gtc".to_string(),
                    }),
                },
                None,
            )
            .await;
        match order {
            Ok(order) => match order {
                ExchangeResponseStatus::Ok(order) => {
                    if let Some(order) = order.data {
                        if !order.statuses.is_empty() {
                            match order.statuses[0].clone() {
                                ExchangeDataStatus::Filled(order) => {
                                    return (amount, order.oid);
                                }
                                ExchangeDataStatus::Resting(order) => {
                                    return (amount, order.oid);
                                }
                                ExchangeDataStatus::Error(e) => {
                                    error!("Error with placing order: {e}")
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            error!("Exchange data statuses is empty when placing order: {order:?}")
                        }
                    } else {
                        error!("Exchange response data is empty when placing order: {order:?}")
                    }
                }
                ExchangeResponseStatus::Err(e) => {
                    error!("Error with placing order: {e}")
                }
            },
            Err(e) => error!("Error with placing order: {e}"),
        }
        (0.0, 0)
    }

    async fn potentially_update(&mut self) {
        let half_spread = (self.latest_mid_price * self.half_spread as f64) / 10000.0;
        // Determine prices to target from the half spread
        let (lower_price, upper_price) = (
            self.latest_mid_price - half_spread,
            self.latest_mid_price + half_spread,
        );
        let (mut lower_price, mut upper_price) = (
            truncate_float(lower_price, self.decimals, true),
            truncate_float(upper_price, self.decimals, false),
        );

        // Rounding optimistically to make our market tighter might cause a weird edge case, so account for that
        if (lower_price - upper_price).abs() < EPSILON {
            lower_price = truncate_float(lower_price, self.decimals, false);
            upper_price = truncate_float(upper_price, self.decimals, true);
        }

        // Determine amounts we can put on the book without exceeding the max absolute position size
        let lower_order_amount = (self.max_absolute_position_size - self.cur_position)
            .min(self.target_liquidity)
            .max(0.0);

        let upper_order_amount = (self.max_absolute_position_size + self.cur_position)
            .min(self.target_liquidity)
            .max(0.0);

        // Determine if we need to cancel the resting order and put a new order up due to deviation
        let lower_change = (lower_order_amount - self.lower_resting.position).abs() > EPSILON
            || bps_diff(lower_price, self.lower_resting.price) > self.max_bps_diff;
        let upper_change = (upper_order_amount - self.upper_resting.position).abs() > EPSILON
            || bps_diff(upper_price, self.upper_resting.price) > self.max_bps_diff;

        // Consider cancelling
        // TODO: Don't block on cancels
        if self.lower_resting.oid != 0 && self.lower_resting.position > EPSILON && lower_change {
            let cancel = self
                .attempt_cancel(self.asset.clone(), self.lower_resting.oid)
                .await;
            // If we were unable to cancel, it means we got a fill, so wait until we receive that event to do anything
            if !cancel {
                return;
            }
            info!("Cancelled buy order: {:?}", self.lower_resting);
        }

        if self.upper_resting.oid != 0 && self.upper_resting.position > EPSILON && upper_change {
            let cancel = self
                .attempt_cancel(self.asset.clone(), self.upper_resting.oid)
                .await;
            if !cancel {
                return;
            }
            info!("Cancelled sell order: {:?}", self.upper_resting);
        }

        // Consider putting a new order up
        if lower_order_amount > EPSILON && lower_change {
            let (amount_resting, oid) = self
                .place_order(self.asset.clone(), lower_order_amount, lower_price, true)
                .await;

            self.lower_resting.oid = oid;
            self.lower_resting.position = amount_resting;
            self.lower_resting.price = lower_price;

            if amount_resting > EPSILON {
                info!(
                    "Buy for {amount_resting} {} resting at {lower_price}",
                    self.asset.clone()
                );
            }
        }

        if upper_order_amount > EPSILON && upper_change {
            let (amount_resting, oid) = self
                .place_order(self.asset.clone(), upper_order_amount, upper_price, false)
                .await;
            self.upper_resting.oid = oid;
            self.upper_resting.position = amount_resting;
            self.upper_resting.price = upper_price;

            if amount_resting > EPSILON {
                info!(
                    "Sell for {amount_resting} {} resting at {upper_price}",
                    self.asset.clone()
                );
            }
        }
    }
}
