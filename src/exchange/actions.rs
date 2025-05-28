use crate::exchange::{cancel::CancelRequest, modify::ModifyRequest, order::OrderRequest};
use alloy::{
    dyn_abi::Eip712Domain,
    primitives::{Address, U256},
    sol,
    sol_types::eip712_domain,
};

use serde::{Deserialize, Serialize};

use super::{cancel::CancelRequestCloid, BuilderInfo};

sol! {
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct UsdSend {
        uint256 signature_chain_id;
        string hyperliquid_chain;
        string destination;
        string amount;
        uint256 time;
    }
}

impl Eip712 for UsdSend {
    fn domain(&self) -> Eip712Domain {
        eip_712_domain(self.signature_chain_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLeverage {
    pub asset: u32,
    pub is_cross: bool,
    pub leverage: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIsolatedMargin {
    pub asset: u32,
    pub is_buy: bool,
    pub ntli: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkOrder {
    pub orders: Vec<OrderRequest>,
    pub grouping: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub builder: Option<BuilderInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkCancel {
    pub cancels: Vec<CancelRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkModify {
    pub modifies: Vec<ModifyRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkCancelCloid {
    pub cancels: Vec<CancelRequestCloid>,
}

pub trait Eip712 {
    fn domain(&self) -> Eip712Domain;
}

fn eip_712_domain(chain_id: U256) -> Eip712Domain {
    let chain_id = chain_id.to::<u64>();
    eip712_domain!(
        name: "HyperliquidSignTransaction",
        version: "1",
        chain_id: chain_id,
    )
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApproveAgentDto {
    pub signature_chain_id: U256,
    pub hyperliquid_chain: String,
    pub agent_address: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    pub nonce: u64,
}

sol! {
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ApproveAgent {
        uint256 signature_chain_id;
        string hyperliquid_chain;
        address agent_address;
        string agent_name;
        #[serde(serialize_with = "serialize_nonce")]
        uint256 nonce;
    }
}

pub fn serialize_nonce<S>(nonce: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u128(nonce.to::<u128>())
}

impl Eip712 for ApproveAgent {
    fn domain(&self) -> Eip712Domain {
        eip_712_domain(self.signature_chain_id)
    }
}

sol! {
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Withdraw3 {
        string hyperliquid_chain;
        uint256 signature_chain_id;
        string amount;
        uint256 time;
        string destination;
    }
}

impl Eip712 for Withdraw3 {
    fn domain(&self) -> Eip712Domain {
        eip_712_domain(self.signature_chain_id)
    }
}

sol! {
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct SpotSend {
        string hyperliquid_chain;
        uint256 signature_chain_id;
        string destination;
        string token;
        string amount;
        uint256 time;
    }
}

impl Eip712 for SpotSend {
    fn domain(&self) -> Eip712Domain {
        eip_712_domain(self.signature_chain_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotUser {
    pub class_transfer: ClassTransfer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClassTransfer {
    pub usdc: u64,
    pub to_perp: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VaultTransfer {
    pub vault_address: Address,
    pub is_deposit: bool,
    pub usd: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetReferrer {
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveBuilderFee {
    pub max_fee_rate: String,
    pub builder: String,
    pub nonce: u64,
    pub signature_chain_id: U256,
    pub hyperliquid_chain: String,
}
