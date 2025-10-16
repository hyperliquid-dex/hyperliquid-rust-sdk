use alloy::{
    dyn_abi::Eip712Domain,
    primitives::{keccak256, Address, B256},
    sol_types::{eip712_domain, SolValue},
};
use serde::{Deserialize, Serialize, Serializer};

use super::{cancel::CancelRequestCloid, BuilderInfo};
use crate::{
    eip712::Eip712,
    exchange::{cancel::CancelRequest, modify::ModifyRequest, order::OrderRequest},
};

fn eip_712_domain(chain_id: u64) -> Eip712Domain {
    eip712_domain! {
        name: "HyperliquidSignTransaction",
        version: "1",
        chain_id: chain_id,
        verifying_contract: Address::ZERO,
    }
}

fn serialize_hex<S>(val: &u64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("0x{val:x}"))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UsdSend {
    #[serde(serialize_with = "serialize_hex")]
    pub signature_chain_id: u64,
    pub hyperliquid_chain: String,
    pub destination: String,
    pub amount: String,
    pub time: u64,
}

impl Eip712 for UsdSend {
    fn domain(&self) -> Eip712Domain {
        eip_712_domain(self.signature_chain_id)
    }

    fn struct_hash(&self) -> B256 {
        let items = (
            keccak256("HyperliquidTransaction:UsdSend(string hyperliquidChain,string destination,string amount,uint64 time)"),
            keccak256(&self.hyperliquid_chain),
            keccak256(&self.destination),
            keccak256(&self.amount),
            &self.time
        );
        keccak256(items.abi_encode())
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveAgent {
    #[serde(serialize_with = "serialize_hex")]
    pub signature_chain_id: u64,
    pub hyperliquid_chain: String,
    pub agent_address: Address,
    pub agent_name: Option<String>,
    pub nonce: u64,
}

impl Eip712 for ApproveAgent {
    fn domain(&self) -> Eip712Domain {
        eip_712_domain(self.signature_chain_id)
    }

    fn struct_hash(&self) -> B256 {
        let items = (
            keccak256("HyperliquidTransaction:ApproveAgent(string hyperliquidChain,address agentAddress,string agentName,uint64 nonce)"),
            keccak256(&self.hyperliquid_chain),
            &self.agent_address,
            keccak256(self.agent_name.as_deref().unwrap_or("")),
            &self.nonce
        );
        keccak256(items.abi_encode())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Withdraw3 {
    #[serde(serialize_with = "serialize_hex")]
    pub signature_chain_id: u64,
    pub hyperliquid_chain: String,
    pub destination: String,
    pub amount: String,
    pub time: u64,
}

impl Eip712 for Withdraw3 {
    fn domain(&self) -> Eip712Domain {
        eip_712_domain(self.signature_chain_id)
    }

    fn struct_hash(&self) -> B256 {
        let items = (
            keccak256("HyperliquidTransaction:Withdraw(string hyperliquidChain,string destination,string amount,uint64 time)"),
            keccak256(&self.hyperliquid_chain),
            keccak256(&self.destination),
            keccak256(&self.amount),
            &self.time,
        );
        keccak256(items.abi_encode())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotSend {
    #[serde(serialize_with = "serialize_hex")]
    pub signature_chain_id: u64,
    pub hyperliquid_chain: String,
    pub destination: String,
    pub token: String,
    pub amount: String,
    pub time: u64,
}

impl Eip712 for SpotSend {
    fn domain(&self) -> Eip712Domain {
        eip_712_domain(self.signature_chain_id)
    }

    fn struct_hash(&self) -> B256 {
        let items = (
            keccak256("HyperliquidTransaction:SpotSend(string hyperliquidChain,string destination,string token,string amount,uint64 time)"),
            keccak256(&self.hyperliquid_chain),
            keccak256(&self.destination),
            keccak256(&self.token),
            keccak256(&self.amount),
            &self.time,
        );
        keccak256(items.abi_encode())
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
pub struct SendAsset {
    #[serde(serialize_with = "serialize_hex")]
    pub signature_chain_id: u64,
    pub hyperliquid_chain: String,
    pub destination: String,
    pub source_dex: String,
    pub destination_dex: String,
    pub token: String,
    pub amount: String,
    pub from_sub_account: String,
    pub nonce: u64,
}

impl Eip712 for SendAsset {
    fn domain(&self) -> Eip712Domain {
        eip_712_domain(self.signature_chain_id)
    }

    fn struct_hash(&self) -> B256 {
        let items = (
            keccak256("HyperliquidTransaction:SendAsset(string hyperliquidChain,string destination,string sourceDex,string destinationDex,string token,string amount,string fromSubAccount,uint64 nonce)"),
            keccak256(&self.hyperliquid_chain),
            keccak256(&self.destination),
            keccak256(&self.source_dex),
            keccak256(&self.destination_dex),
            keccak256(&self.token),
            keccak256(&self.amount),
            keccak256(&self.from_sub_account),
            &self.nonce,
        );
        keccak256(items.abi_encode())
    }
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
pub struct EvmUserModify {
    pub using_big_blocks: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveBuilderFee {
    #[serde(serialize_with = "serialize_hex")]
    pub signature_chain_id: u64,
    pub hyperliquid_chain: String,
    pub builder: Address,
    pub max_fee_rate: String,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleCancel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClaimRewards;

impl Eip712 for ApproveBuilderFee {
    fn domain(&self) -> Eip712Domain {
        eip_712_domain(self.signature_chain_id)
    }

    fn struct_hash(&self) -> B256 {
        let items = (
            keccak256("HyperliquidTransaction:ApproveBuilderFee(string hyperliquidChain,string maxFeeRate,address builder,uint64 nonce)"),
            keccak256(&self.hyperliquid_chain),
            keccak256(&self.max_fee_rate),
            &self.builder,
            &self.nonce,
        );
        keccak256(items.abi_encode())
    }
}
