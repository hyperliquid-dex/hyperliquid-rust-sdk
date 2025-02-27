use crate::{
    errors::Error,
    helpers::{float_to_string_for_hashing, uuid_to_hex_string},
    prelude::*,
};
use ethers::signers::LocalWallet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Limit {
    pub tif: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    pub is_market: bool,
    pub trigger_px: String,
    pub tpsl: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Order {
    Limit(Limit),
    Trigger(Trigger),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
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
    pub wallet: Option<&'a LocalWallet>,
}

#[derive(Debug)]
pub struct MarketCloseParams<'a> {
    pub asset: &'a str,
    pub sz: Option<f64>,
    pub px: Option<f64>,
    pub slippage: Option<f64>,
    pub cloid: Option<Uuid>,
    pub wallet: Option<&'a LocalWallet>,
}

#[derive(Debug)]
pub enum ClientOrder {
    Limit(ClientLimit),
    Trigger(ClientTrigger),
}

/// Asset: the asset attempting to be bought, represented by a string such as BASE/QUOTE
/// is_buy: whether we are buying or selling the base currency base on the Side type - either bid or ask.
/// reduce_only: reduce only orders adjust or reduce current position to match the size of the current open position.
/// limit_px: The price willing to bid or ask, here we round up if selling to get maximum price and down if we are bidding to make sure
/// sz: The amount of the asset wishing to be purchased, this is always rounded down if bidding to make sure we don't over spend and up if we are asking to make sure we sell whole tokens.
/// Limit price and order size rely on the fork as initially this was f64 which we chose to remove due to precision errors
/// cloid: Central limit order id, for HyperLiquid this is optional but in order to query and track transactions, thus providing meaning this will be the order id designated from the OMS.
/// order_type - tif: the order type wishing to be performed, this system will always support Ioc - Immediate or Cancel.
#[derive(Debug)]
pub struct ClientOrderRequest<T = f64> {
    pub asset: String,
    pub is_buy: bool,
    pub reduce_only: bool,
    pub limit_px: T,
    pub sz: T,
    pub cloid: Option<Uuid>,
    pub order_type: ClientOrder,
}

impl ClientOrderRequest<f64>
{
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

impl ClientOrderRequest<String> {
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
            limit_px: self.limit_px,
            sz: self.sz,
            order_type,
            cloid,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{exchange::order::{Limit, OrderRequest}, Order};

    use super::{ClientLimit, ClientOrder, ClientOrderRequest};
    use std::collections::HashMap;

    #[test]
    fn test_f64_client_order_request() {

        let test_asset = "XYZTWO/USDC".to_string();
        let coin_to_asset: &mut HashMap<std::string::String, u32> = &mut HashMap::new();
        coin_to_asset.insert(test_asset.clone(), 123);

        let f64_client_order_under_test = ClientOrderRequest {
            asset: test_asset.clone(),
            is_buy: true,
            reduce_only: false,
            limit_px: 2.0202,
            sz: 1.0001,
            cloid: None,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Gtc".to_string(),
            }),
        };

        let test_result = f64_client_order_under_test.convert(coin_to_asset);

        assert_eq!(
            test_result.unwrap(),
            OrderRequest {
                asset: 123,
                is_buy: true,
                limit_px: "2.0202".to_string(),
                sz: "1.0001".to_string(),
                reduce_only: false,
                order_type: Order::Limit(Limit { tif: "Gtc".to_string() }),
                cloid: None
            }
        );
    }

    #[test]
    fn test_string_client_order_request() {
        let coin_to_asset: &mut HashMap<std::string::String, u32> = &mut HashMap::new();
        coin_to_asset.insert("XYZTWO/USDC".to_string(), 123);

        let string_client_order_under_test = ClientOrderRequest::<String> {
            asset: "XYZTWO/USDC".to_string(),
            is_buy: true,
            reduce_only: false,
            limit_px: "2.000".to_string(),
            sz: "3.0000".to_string(),
            cloid: None,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Gtc".to_string(),
            }),
        };

        let test_result = string_client_order_under_test.convert(coin_to_asset);

        assert_eq!(
            test_result.unwrap(),
            OrderRequest {
                asset: 123,
                is_buy: true,
                limit_px: "2.000".to_string(),
                sz: "3.0000".to_string(),
                reduce_only: false,
                order_type: Order::Limit(Limit { tif: "Gtc".to_string() }),
                cloid: None
            }
        );
    }
}
