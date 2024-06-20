use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::Signer;
use ethers::signers::{LocalWallet, Wallet};
use hyperliquid_rust_sdk::{
    bps_diff, truncate_float, BaseUrl, ClientCancelRequest, ClientLimit, ClientOrder,
    ClientOrderRequest, ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus, InfoClient,
    Message, Subscription, TradeInfo, EPSILON,
};
use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info, warn};

/// A lightweight wrapper around the HyperLiquid clients
struct HyperLiquidClient {
    ro: InfoClient,
    // Client for informational queries/read-only
    rw: ExchangeClient, // Client for mutating exchange state
}

/// The core market making engine
struct Maker {
    client: HyperLiquidClient,
    // Connections to the exchange
    wallet: LocalWallet,
    // The wallet for the maker to use
    markets: HashMap<String, Market>,
}

/// State associated with a given market
#[derive(Debug, Clone)]
struct Market {
    asset: &'static str,
    // The ticker for this market
    mid: f64,
    // The current midpoint of the market
    inventory: f64,
    // The maker's current inventory in this market
    target_liquidity: f64,
    // The desired liquidity to provide to each side of the book
    half_spread: u16,
    // The distance from midpoint to take on each side
    max_bps_diff: u16,
    // The maximum bps move in the market midpoint before updating orders
    max_absolute_position_size: f64,
    // The maximum inventory the mm should take in any circumstance, directionally
    decimals: u32,
    // The number of decimals for the market.
    orders: HashMap<u64, Order>,
    // Map of open orders on this market
    resting_bid_order: Option<u64>,
    // The resting bid order if any
    resting_ask_order: Option<u64>,  // The resting ask order if any
}

/// State associated with an order outstanding on the book
#[derive(Debug, Copy, Clone)]
struct Order {
    position: f64,
    price: f64,
}

impl Maker {
    /// Write methods

    /// Attempts to cancel an order
    ///
    /// Generally returns error when the order has already been filled.
    async fn attempt_cancel(&self, asset: &str, oid: u64) -> Result<()> {
        // Send cancellation request to the exchange
        let cancel = self
            .client
            .rw
            .cancel(
                ClientCancelRequest {
                    asset: asset.to_string(),
                    oid,
                },
                None,
            )
            .await;

        // Check if the cancellation succeeded
        // The endpoint design is pretty horrible here and so is the SDK,
        // the status is always ok, and the errors are weakly typed,
        // not documented, and hidden within 3 levels of json.
        // An SDK should abstract all of this and return Ok or Error with
        // proper details.
        match cancel {
            Ok(cancel) => match cancel {
                ExchangeResponseStatus::Ok(cancel) => {
                    if let Some(cancel) = cancel.data {
                        if !cancel.statuses.is_empty() {
                            match cancel.statuses[0].clone() {
                                ExchangeDataStatus::Success => {
                                    return Ok(());
                                }
                                ExchangeDataStatus::Error(e) => {
                                    warn!("Error with cancelling: {e}")
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
                ExchangeResponseStatus::Err(e) => warn!("Error with cancelling: {e}"),
            },
            Err(e) => warn!("Error with cancelling: {e}"),
        }
        Err(anyhow!("Failed to cancel order: {}", oid))
    }

    /// Places an order for a given asset
    async fn place_order(
        &self,
        asset: String,
        amount: f64,
        price: f64,
        is_buy: bool,
    ) -> Result<(f64, u64)> {
        let order = self
            .client
            .rw
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
                                ExchangeDataStatus::Filled(order) => Ok((amount, order.oid)),
                                ExchangeDataStatus::Resting(order) => Ok((amount, order.oid)),
                                ExchangeDataStatus::Error(e) => {
                                    Err(anyhow!("Error with placing order: {e}"))
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            Err(anyhow!(
                                "Exchange data statuses is empty when placing order: {order:?}"
                            ))
                        }
                    } else {
                        Err(anyhow!(
                            "Exchange response data is empty when placing order: {order:?}"
                        ))
                    }
                }
                ExchangeResponseStatus::Err(e) => Err(anyhow!("Error with placing order: {e}")),
            },
            Err(e) => Err(anyhow!("Error with placing order: {e}")),
        }
    }

    async fn potentially_update_market(&mut self, asset: &String) {
        // Get a reference to the market
        let market = self.markets.get(asset).unwrap();

        // Run the simplistic model for the market

        // We calculate the half-spread amount
        let half_spread = (market.mid * market.half_spread as f64) / 10000.0;

        // Determine prices to target from the half-spread
        let (bid_price, ask_price) = (market.mid - half_spread, market.mid + half_spread);
        let (mut bid_price, mut ask_price) = (
            truncate_float(bid_price, market.decimals, true),
            truncate_float(ask_price, market.decimals, false),
        );

        // Rounding optimistically to make our market tighter might cause a weird edge case, so account for that
        if (bid_price - ask_price).abs() < EPSILON {
            bid_price = truncate_float(bid_price, market.decimals, false);
            ask_price = truncate_float(ask_price, market.decimals, true);
        }

        // Determine amounts we can put on the book without exceeding the max absolute position size

        // In some very simplistic way this creates an inventory control
        let bid_order_amount = (market.max_absolute_position_size - market.inventory)
            .min(market.target_liquidity)
            .max(0.0);

        let ask_order_amount = (market.max_absolute_position_size + market.inventory)
            .min(market.target_liquidity)
            .max(0.0);

        debug!("Model Bid {bid_order_amount} {} @ ${bid_price} / Model Ask {ask_order_amount} {} @ ${ask_price}", market.asset, market.asset);

        // Get resting orders if any
        let bid_oid = market.resting_bid_order;
        let ask_oid = market.resting_ask_order;

        let mut new_bid_oid = None;
        let mut new_ask_oid = None;
        let mut new_bid_order = None;
        let mut new_ask_order = None;

        // TODO(If the `ExchangeClient` was Clone + Send + Sync, these orders and cancellations could be placed in parallel)
        // The exchange client would need to be cloned or copied as a ref for each action taken in parallel

        // Determine if any updates are needed
        if let Some(oid) = bid_oid {
            // Update existing bid order if needed
            let resting_bid_order = market.orders.get(&oid).unwrap();
            if (bid_order_amount - resting_bid_order.position).abs() > EPSILON
                || bps_diff(bid_price, resting_bid_order.price) > market.max_bps_diff
            {
                // Enqueue cancellation
                match self.attempt_cancel(market.asset, oid).await {
                    Ok(_) => {
                        info!("Cancelled: Bid order {} on {} ", oid, market.asset);
                    }
                    Err(_) => {
                        // If we were unable to cancel, it means we got a full fill
                        return;
                    }
                };
                // Enqueue new order
                match self
                    .place_order(market.asset.to_string(), bid_order_amount, bid_price, true)
                    .await
                {
                    Ok(order_result) => {
                        new_bid_oid = Some(order_result.1);
                        new_bid_order = Some(Order {
                            position: bid_order_amount,
                            price: bid_price,
                        });
                        info!(
                            "Placed: Bid for {bid_order_amount} {} resting at ${bid_price}",
                            market.asset
                        );
                    }
                    Err(_) => {
                        warn!(
                            "Failed: to place resting bid order for {} {} @ ${}",
                            bid_order_amount,
                            market.asset.to_string(),
                            bid_price
                        )
                    }
                };
            }
        } else if bid_order_amount > EPSILON {
            // Enqueue new bid order
            match self
                .place_order(market.asset.to_string(), bid_order_amount, bid_price, true)
                .await
            {
                Ok(order_result) => {
                    new_bid_oid = Some(order_result.1);
                    new_bid_order = Some(Order {
                        position: bid_order_amount,
                        price: bid_price,
                    });
                    info!(
                        "Placed: Bid for {bid_order_amount} {} resting at ${bid_price}",
                        market.asset
                    );
                }
                Err(_) => {
                    warn!(
                        "Failed: to place resting bid order for {} {} @ ${}",
                        bid_order_amount,
                        market.asset.to_string(),
                        bid_price
                    );
                    return;
                }
            };
        }

        if let Some(oid) = ask_oid {
            // Update existing ask order if needed
            let resting_ask_order = market.orders.get(&oid).unwrap();
            if (ask_order_amount - resting_ask_order.position).abs() > EPSILON
                || bps_diff(ask_price, resting_ask_order.price) > market.max_bps_diff
            {
                // Enqueue cancellation
                match self.attempt_cancel(market.asset, oid).await {
                    Ok(_) => {
                        info!("Cancelled: Ask order {} on {} ", oid, market.asset);
                    }
                    Err(_) => {
                        // If we were unable to cancel, it means we got a full fill
                        return;
                    }
                };
                // Enqueue new ask order
                match self
                    .place_order(market.asset.to_string(), ask_order_amount, ask_price, false)
                    .await
                {
                    Ok(order_result) => {
                        new_ask_oid = Some(order_result.1);
                        new_ask_order = Some(Order {
                            position: ask_order_amount,
                            price: ask_price,
                        });
                        info!(
                            "Placed: Ask for {ask_order_amount} {} resting at ${ask_price}",
                            market.asset
                        );
                    }
                    Err(_) => {
                        warn!(
                            "Failed: to place resting ask order for {} {} @ ${}",
                            ask_order_amount,
                            market.asset.to_string(),
                            ask_price
                        );
                        return;
                    }
                };
            }
        } else if ask_order_amount > EPSILON {
            // Enqueue new ask order
            match self
                .place_order(market.asset.to_string(), ask_order_amount, ask_price, false)
                .await
            {
                Ok(order_result) => {
                    new_ask_oid = Some(order_result.1);
                    new_ask_order = Some(Order {
                        position: ask_order_amount,
                        price: ask_price,
                    });
                    info!(
                        "Placed: Ask for {ask_order_amount} {} resting at ${ask_price}",
                        market.asset
                    );
                }
                Err(_) => {
                    warn!(
                        "Failed to place resting ask order for {} {} @ ${}",
                        ask_order_amount,
                        market.asset.to_string(),
                        ask_price
                    );
                    return;
                }
            };
        }

        // Update state
        let market = self.markets.get_mut(asset).unwrap();
        if new_bid_oid.is_some() {
            market.resting_bid_order = new_bid_oid;
            market
                .orders
                .insert(new_bid_oid.unwrap(), new_bid_order.unwrap());
        }
        if new_ask_oid.is_some() {
            market.resting_ask_order = new_ask_oid;
            market
                .orders
                .insert(new_ask_oid.unwrap(), new_ask_order.unwrap());
        }
    }

    async fn potentially_update_markets(
        &mut self,
        markets_to_potentially_update: &mut VecDeque<String>,
    ) {
        while let Some(asset) = markets_to_potentially_update.pop_front() {
            self.potentially_update_market(&asset).await;
        }
    }

    /// Low-level event handlers.

    /// Handle market mid price update events
    async fn handle_mid(&mut self, mid: &(String, String)) {
        // Check if the market is enabled
        match self.markets.get_mut(&mid.0) {
            None => {}
            Some(market) => {
                market.mid = mid.1.parse().unwrap();
                info!("Market: midpoint for {} @ {}", mid.0, mid.1);
            }
        }
    }

    /// Handle user order fill events
    async fn handle_fill(&mut self, fill: &TradeInfo) {
        let markets = self.markets.get_mut(&fill.coin);
        match markets {
            None => {
                panic!("Market not found! We have fills coming in on an unsupported market.")
            }
            Some(market) => {
                // Get the amount and price of the fill
                let amount: f64 = fill.sz.parse().unwrap();
                let price: f64 = fill.px.parse().unwrap();

                // Update order details
                let oid: u64 = fill.oid;
                match market.orders.get_mut(&oid) {
                    None => {}
                    Some(order) => {
                        debug!("Cleaning up order");
                        order.position -= amount;

                        // Delete order from map if fully filled
                        if order.position <= EPSILON {
                            market.orders.remove(&oid);
                            // Set resting order to none if resting order was filled
                            if market.resting_bid_order == Some(oid) {
                                debug!("Removing bid");
                                market.resting_bid_order = None;
                            } else if market.resting_ask_order == Some(oid) {
                                debug!("Removing ask");
                                market.resting_ask_order = None;
                            }
                        }
                    }
                };

                // Update inventory details
                if fill.side.eq("B") {
                    market.inventory += amount;
                    info!(
                        "Fill: Bid for {} {} @ {} order {}",
                        amount, fill.coin, price, oid
                    );
                } else {
                    market.inventory -= amount;
                    info!(
                        "Fill: Ask for {} {} @ {} order {}",
                        amount, fill.coin, price, oid
                    );
                }
            }
        };
    }

    /// Setup subscriptions to the exchange which the market maker cares about
    async fn setup_subscriptions(&mut self, sender: UnboundedSender<Message>) -> Result<()> {
        // Subscribe to UserEvents for fills
        self.client
            .ro
            .subscribe(
                Subscription::UserEvents {
                    user: self.wallet.address(),
                },
                sender.clone(),
            )
            .await
            .unwrap();

        // Subscribe to AllMids to get the latest market midpoint prices
        self.client
            .ro
            .subscribe(Subscription::AllMids, sender.clone())
            .await
            .unwrap();

        Ok(())
    }

    async fn event_listener(&mut self, receiver: &mut UnboundedReceiver<Message>) {
        loop {
            let mut markets_to_update: VecDeque<String> = VecDeque::new();
            // Wait for a new message on the channel
            let message = receiver.recv().await.unwrap();
            debug!("Got new message on channel  {:?}", message);

            // Process the message
            match message {
                Message::AllMids(all_mids) => {
                    let all_mids = all_mids.data.mids;
                    for mid in all_mids {
                        // Check if the market is enabled
                        match self.markets.get(&mid.0) {
                            None => {}
                            Some(_) => {
                                self.handle_mid(&mid).await;
                                markets_to_update.push_back(mid.0)
                            }
                        }
                    }
                }
                Message::User(user_events) => {
                    let fills = user_events.data.fills;
                    for fill in fills {
                        self.handle_fill(&fill).await;
                        markets_to_update.push_back(fill.coin)
                    }
                }
                _ => {
                    panic!("Unsupported message type");
                }
            }
            self.potentially_update_markets(&mut markets_to_update)
                .await;
        }
    }

    async fn run(&mut self) -> Result<()> {
        // Set up a channel for message passing with the exchange
        let (sender, mut receiver) = unbounded_channel();

        // Setup subscriptions
        self.setup_subscriptions(sender.clone()).await.unwrap();

        // The event listener handles events which come in from the exchange
        self.event_listener(&mut receiver).await;

        Err(anyhow!("Maker exited unexpectedly"))
    }
}

/// The entrypoint to run the Mercator
#[tokio::main]
async fn main() -> Result<()> {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::FmtSubscriber::new();

    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber)?;

    // A lightweight CLI for parsing arguments, flags, and environment variables
    let matches = Command::new("market_maker")
        .author("0xAlcibiades")
        .version("0.0.1")
        .about("An example market maker for Hyperliquid.")
        .arg(Arg::new("private_key")
            .env("PRIVATE_KEY")
            .required(true)
            // TODO(Use secrets or another similar crate to ensure this never leaks)
            .value_parser(clap::value_parser!(Wallet<SigningKey>))
            .help("Valid private key of the wallet to trade with.")
        )
        .get_matches();

    // Safe to unwrap here as the parser already caught it being valid.
    // We clone here so that we have an owned instance.
    let wallet = matches
        .get_one::<Wallet<SigningKey>>("private_key")
        .unwrap()
        .clone();

    // Now that we have a wallet, we connect to the exchange

    // Getting an info client for read-only
    let info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap();

    // and an exchange client for read-write
    let exchange_client =
        ExchangeClient::new(None, wallet.clone(), Some(BaseUrl::Mainnet), None, None)
            .await
            .unwrap();

    // Wrap up the two clients nicely
    let client = HyperLiquidClient {
        ro: info_client,
        rw: exchange_client,
    };

    let mut market_states: HashMap<String, Market> = HashMap::new();
    market_states.insert(
        "kSHIB".to_string(),
        Market {
            asset: "kSHIB",
            mid: 0.0,
            inventory: 0.0,
            target_liquidity: 100000.0,
            half_spread: 10,
            max_bps_diff: 5,
            max_absolute_position_size: 150000.0,
            decimals: 6,
            resting_bid_order: None,
            resting_ask_order: None,
            orders: Default::default(),
        },
    );

    // Instantiate a maker
    let mut maker = Maker {
        client,
        wallet,
        markets: market_states,
    };

    // Run the maker
    let _ = maker.run().await;

    // This should have run indefinitely, so we exit error if we reach this point,
    // however, this isn't a production grade maker yet, i.e., there is no exponential
    // backoff and retry or health checks on the exchange connections.
    Err(anyhow!("Maker exited unexpectedly!"))
}
