use crate::{
    errors::Error,
    helpers::{float_to_int_for_hashing, float_to_string_for_hashing},
    prelude::*,
};
use ethers::types::H160;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone)]
pub(crate) struct Limit {
    pub(crate) tif: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Trigger {
    pub(crate) trigger_px: String,
    pub(crate) is_market: bool,
    pub(crate) tpsl: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum OrderType {
    Limit(Limit),
    Trigger(Trigger),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BulkOrderRequest {
    pub(crate) grouping: String,
    pub(crate) orders: Vec<OrderRequest>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrderRequest {
    pub(crate) asset: u32,
    pub(crate) is_buy: bool,
    pub(crate) reduce_only: bool,
    pub(crate) limit_px: String,
    pub(crate) sz: String,
    pub(crate) order_type: OrderType,
}

pub struct ClientLimit {
    pub tif: String,
}

pub struct ClientTrigger {
    pub trigger_px: f64,
    pub is_market: bool,
    pub tpsl: String,
}

pub enum ClientOrderType {
    Limit(ClientLimit),
    Trigger(ClientTrigger),
}
pub struct ClientOrderRequest {
    pub asset: String,
    pub is_buy: bool,
    pub reduce_only: bool,
    pub limit_px: f64,
    pub sz: f64,
    pub order_type: ClientOrderType,
}

impl ClientOrderRequest {
    pub(crate) fn convert(self, coin_to_asset: &HashMap<String, u32>) -> Result<OrderRequest> {
        let order_type = match self.order_type {
            ClientOrderType::Limit(limit) => OrderType::Limit(Limit { tif: limit.tif }),
            ClientOrderType::Trigger(trigger) => OrderType::Trigger(Trigger {
                trigger_px: float_to_string_for_hashing(trigger.trigger_px),
                is_market: trigger.is_market,
                tpsl: trigger.tpsl,
            }),
        };
        let &asset = coin_to_asset.get(&self.asset).ok_or(Error::AssetNotFound)?;

        Ok(OrderRequest {
            asset,
            is_buy: self.is_buy,
            reduce_only: self.reduce_only,
            limit_px: float_to_string_for_hashing(self.limit_px),
            sz: float_to_string_for_hashing(self.sz),
            order_type,
        })
    }
    pub(crate) fn create_hashable_tuple(
        &self,
        coin_to_asset: &HashMap<String, u32>,
    ) -> Result<(u32, bool, u64, u64, bool, u8, u64)> {
        let hashed_order_type = self.order_type.get_type()?;
        let &asset = coin_to_asset.get(&self.asset).ok_or(Error::AssetNotFound)?;
        Ok((
            asset,
            self.is_buy,
            float_to_int_for_hashing(self.limit_px),
            float_to_int_for_hashing(self.sz),
            self.reduce_only,
            hashed_order_type.0,
            hashed_order_type.1,
        ))
    }
}

impl ClientOrderType {
    pub(crate) fn get_type(&self) -> Result<(u8, u64)> {
        match self {
            ClientOrderType::Limit(limit) => match limit.tif.as_str() {
                "Gtc" => Ok((2, 0)),
                "Alo" => Ok((1, 0)),
                "Ioc" => Ok((3, 0)),
                _ => Err(Error::OrderTypeNotFound),
            },
            ClientOrderType::Trigger(trigger) => match (trigger.is_market, trigger.tpsl.as_str()) {
                (true, "tp") => Ok((4, float_to_int_for_hashing(trigger.trigger_px))),
                (false, "tp") => Ok((5, float_to_int_for_hashing(trigger.trigger_px))),
                (true, "sl") => Ok((6, float_to_int_for_hashing(trigger.trigger_px))),
                (false, "sl") => Ok((7, float_to_int_for_hashing(trigger.trigger_px))),
                _ => Err(Error::OrderTypeNotFound),
            },
        }
    }
}
