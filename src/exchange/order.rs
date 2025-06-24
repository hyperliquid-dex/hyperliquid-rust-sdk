use crate::{
    errors::Error,
    helpers::{float_to_string_for_hashing, uuid_to_hex_string},
    prelude::*,
};
use alloy::signers::local::PrivateKeySigner;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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

#[derive(Debug)]
pub struct ClientLimit {
    pub tif: String,
}

#[derive(Debug)]
pub struct ClientTrigger {
    pub is_market: bool,
    pub trigger_px: f64,
    pub tpsl: String,
}

#[derive(Debug)]
pub struct MarketOrderParams<'a> {
    pub asset: &'a str,
    pub is_buy: bool,
    pub sz: f64,
    pub px: Option<f64>,
    pub slippage: Option<f64>,
    pub cloid: Option<Uuid>,
    pub wallet: Option<&'a PrivateKeySigner>,
}

#[derive(Debug)]
pub struct MarketCloseParams<'a> {
    pub asset: &'a str,
    pub sz: Option<f64>,
    pub px: Option<f64>,
    pub slippage: Option<f64>,
    pub cloid: Option<Uuid>,
    pub wallet: Option<&'a PrivateKeySigner>,
}

#[derive(Debug)]
pub enum ClientOrder {
    Limit(ClientLimit),
    Trigger(ClientTrigger),
}

#[derive(Debug)]
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
