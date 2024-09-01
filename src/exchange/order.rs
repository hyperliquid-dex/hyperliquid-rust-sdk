use crate::{
    errors::Error,
    helpers::{float_to_string_for_hashing, uuid_to_hex_string},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;



// used to work out a order price that will be agressed (hit/lifted)
fn aggress_price(mid_price: f64, is_buy: bool) -> f64 {
    const SLIPPAGE: f64 = 0.05;
    let c = if is_buy { SLIPPAGE } else { -SLIPPAGE };
    mid_price * 1.0 + c
}

// rounds to the nearest tick price based 0.5 tick size
fn round_to_nearest_tick(value: f64) -> f64 {
    (value * 2.0).round() / 2.0
}

// create a market order by creating a limit order that aggresses the book
pub(crate) fn market_order(asset: String, mid_price: f64, is_buy: bool, qty: f64, cloid: Option<Uuid>) -> ClientOrderRequest {
    let limit_px = aggress_price(mid_price, is_buy);

    let tick_price = round_to_nearest_tick(limit_px);
    ClientOrderRequest {
        asset,
        is_buy,
        reduce_only: false,
        limit_px: tick_price,
        sz: qty,
        cloid,
        order_type: ClientOrder::Limit(ClientLimit {
            tif: "Ioc".to_string(),
        }),
    }
}


#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Limit {
    pub tif: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    pub is_market: bool,
    pub trigger_px: String,
    pub tpsl: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Order {
    Limit(Limit),
    Trigger(Trigger),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    #[serde(rename = "a", alias = "asset")]
    pub asset: u32,
    #[serde(rename = "b", alias = "isBuy")]
    pub is_buy: bool,
    #[serde(rename = "p", alias = "limitPx")]
    pub limit_px: String,
    #[serde(rename = "s", alias = "sz")]
    pub sz: String,
    #[serde(rename = "r", alias = "reduceOnly", default)]
    pub reduce_only: bool,
    #[serde(rename = "t", alias = "orderType")]
    pub order_type: Order,
    #[serde(rename = "c", alias = "cloid", skip_serializing_if = "Option::is_none")]
    pub cloid: Option<String>,
}

pub struct ClientLimit {
    pub tif: String,
}

pub struct ClientTrigger {
    pub is_market: bool,
    pub trigger_px: f64,
    pub tpsl: String,
}

pub enum ClientOrder {
    Limit(ClientLimit),
    Trigger(ClientTrigger),
}
pub struct ClientOrderRequest {
    pub asset: String,
    pub is_buy: bool,
    pub reduce_only: bool,
    pub limit_px: f64,
    pub sz: f64,
    pub cloid: Option<Uuid>,
    pub order_type: ClientOrder,
}

impl ClientOrderRequest {
    pub(crate) fn convert(self, coin_to_asset: &HashMap<String, u32>) -> Result<OrderRequest> {
        let order_type = match self.order_type {
            ClientOrder::Limit(limit) => Order::Limit(Limit { tif: limit.tif }),
            ClientOrder::Trigger(trigger) => Order::Trigger(Trigger {
                trigger_px: float_to_string_for_hashing(trigger.trigger_px),
                is_market: trigger.is_market,
                tpsl: trigger.tpsl,
            }),
        };
        let &asset = coin_to_asset.get(&self.asset).ok_or(Error::AssetNotFound)?;

        let cloid = self.cloid.map(uuid_to_hex_string);

        Ok(OrderRequest {
            asset,
            is_buy: self.is_buy,
            reduce_only: self.reduce_only,
            limit_px: float_to_string_for_hashing(self.limit_px),
            sz: float_to_string_for_hashing(self.sz),
            order_type,
            cloid,
        })
    }
}
