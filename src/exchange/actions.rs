use crate::{
    exchange::{cancel::CancelRequest, order::OrderRequest},
    signature::agent::mainnet::Agent,
};
use ethers::types::H160;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct UsdcTransfer {
    pub(crate) chain: String,
    pub(crate) payload: serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateLeverage {
    pub(crate) asset: u32,
    pub(crate) is_cross: bool,
    pub(crate) leverage: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateIsolatedMargin {
    pub(crate) asset: u32,
    pub(crate) is_buy: bool,
    pub(crate) ntli: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BulkOrder {
    pub(crate) grouping: String,
    pub(crate) orders: Vec<OrderRequest>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BulkCancel {
    pub(crate) cancels: Vec<CancelRequest>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentConnect {
    pub(crate) chain: String,
    pub(crate) agent: Agent,
    pub(crate) agent_address: H160,
}
