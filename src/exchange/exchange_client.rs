use crate::{
    exchange::{
        actions::{
            ApproveAgent, ApproveBuilderFee, BulkCancel, BulkModify, BulkOrder, SetReferrer,
            UpdateIsolatedMargin, UpdateLeverage, UsdSend,
        },
        cancel::{CancelRequest, CancelRequestCloid},
        modify::{ClientModifyRequest, ModifyRequest},
        ClientCancelRequest, ClientOrderRequest,
    },
    helpers::{next_nonce, uuid_to_hex_string},
    prelude::*,
    signature::create_signature::encode_l1_action,
    BulkCancelCloid, Error, SendAsset,
};
use crate::{ClassTransfer, SpotSend, VaultTransfer, Withdraw3};
use ethers::types::{H160, H256};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{cancel::ClientCancelRequestCloid, dtos::MessageResponse};
use super::{
    dtos::SpotTransferRequest,
    order::{MarketOrderParams, SetTpSlParams},
};
use super::{BuilderInfo, ClientLimit, ClientOrder};

#[cfg(not(feature = "testnet"))]
const HYPERLIQUID_CHAIN: &str = "Mainnet";

#[cfg(feature = "testnet")]
const HYPERLIQUID_CHAIN: &str = "Testnet";

#[cfg(not(feature = "testnet"))]
const SIGNATURE_CHAIN_ID: i64 = 999;

#[cfg(feature = "testnet")]
const SIGNATURE_CHAIN_ID: i64 = 998;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Actions {
    UsdSend(UsdSend),
    UpdateLeverage(UpdateLeverage),
    UpdateIsolatedMargin(UpdateIsolatedMargin),
    Order(BulkOrder),
    Cancel(BulkCancel),
    CancelByCloid(BulkCancelCloid),
    BatchModify(BulkModify),
    ApproveAgent(ApproveAgent),
    Withdraw3(Withdraw3),
    VaultTransfer(VaultTransfer),
    SpotSend(SpotSend),
    SetReferrer(SetReferrer),
    ApproveBuilderFee(ApproveBuilderFee),
    SendAsset(SendAsset),
    UsdClassTransfer(ClassTransfer),
}

impl Actions {
    fn hash(&self, timestamp: u64, vault_address: Option<H160>) -> Result<H256> {
        let mut bytes =
            rmp_serde::to_vec_named(self).map_err(|e| Error::RmpParse(e.to_string()))?;
        bytes.extend(timestamp.to_be_bytes());
        if let Some(vault_address) = vault_address {
            bytes.push(1);
            bytes.extend(vault_address.to_fixed_bytes());
        } else {
            bytes.push(0);
        }
        Ok(H256(ethers::utils::keccak256(bytes)))
    }
}

pub struct HashGenerator {}

impl HashGenerator {
    pub async fn usdc_transfer(amount: &str, destination: &str) -> Result<Value> {
        let timestamp = next_nonce();

        let usd_send = UsdSend {
            signature_chain_id: 421614.into(),
            hyperliquid_chain: HYPERLIQUID_CHAIN.to_string(),
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: timestamp,
        };
        let action = serde_json::to_value(Actions::UsdSend(usd_send))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
    }

    pub async fn approve_builder_fee(
        builder: String,
        max_fee_rate: String,
    ) -> Result<MessageResponse> {
        let timestamp = next_nonce();
        let action = ApproveBuilderFee {
            builder,
            max_fee_rate,
            nonce: timestamp,
            signature_chain_id: SIGNATURE_CHAIN_ID.into(),
            hyperliquid_chain: HYPERLIQUID_CHAIN.to_string(),
        };

        let message = action
            .eip712_signing_hash()
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(MessageResponse {
            action: Actions::ApproveBuilderFee(action),
            message,
            nonce: timestamp,
        })
    }

pub async fn send_asset(
        token: String,
        source_dex: String,
        destination_dex: String,
        destination: String,
        amount: String,
        from_sub_account: String,
    ) -> Result<MessageResponse> {
        let timestamp = next_nonce();
        let perp_dex_class_transfer = SendAsset {
            token,
            source_dex,
            destination_dex,
            destination,
            amount: amount.to_string(),
            from_sub_account,
            nonce: timestamp,
            hyperliquid_chain: HYPERLIQUID_CHAIN.to_string(),
            signature_chain_id: SIGNATURE_CHAIN_ID.into(),
        };
        let message = perp_dex_class_transfer
            .eip712_signing_hash()
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(MessageResponse {
            action: Actions::SendAsset(perp_dex_class_transfer),
            message,
            nonce: timestamp,
        })
    }

    pub async fn class_transfer(amount: String, to_perp: bool) -> Result<MessageResponse> {
        let timestamp = next_nonce();

        let class_transfer = ClassTransfer {
            amount,
            to_perp,
            nonce: timestamp,
            hyperliquid_chain: HYPERLIQUID_CHAIN.to_string(),
            signature_chain_id: SIGNATURE_CHAIN_ID.into(),
        };

        // Use EIP-712 signing directly (not L1 action hashing)
        let message = class_transfer
            .eip712_signing_hash()
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(MessageResponse {
            action: Actions::UsdClassTransfer(class_transfer),
            message,
            nonce: timestamp,
        })
    }

    pub async fn vault_transfer(
        is_deposit: bool,
        usd: u64,
        vault_address: Option<H160>,
    ) -> Result<Value> {
        let vault_address = vault_address.ok_or(Error::VaultAddressNotFound)?;

        let action = Actions::VaultTransfer(VaultTransfer {
            vault_address,
            is_deposit,
            usd,
        });
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
    }

    pub async fn market_open(params: MarketOrderParams) -> Result<MessageResponse> {
        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only: params.reduce_only,
            limit_px: params.px.parse::<f64>().unwrap(),
            sz: params.sz.parse::<f64>().unwrap(),
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Ioc".to_string(),
            }),
        };

        Self::get_message_for_order(vec![order])
    }

    pub async fn limit_open(params: MarketOrderParams) -> Result<MessageResponse> {
        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only: params.reduce_only,
            limit_px: params.px.parse::<f64>().unwrap(),
            sz: params.sz.parse::<f64>().unwrap(),
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Gtc".to_string(),
            }),
        };

        Self::get_message_for_order(vec![order])
    }

    pub async fn set_tp_sl(params: SetTpSlParams) -> Result<MessageResponse> {
        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only: params.reduce_only,
            limit_px: params.px.parse::<f64>().unwrap(),
            sz: params.sz.parse::<f64>().unwrap(),
            cloid: params.cloid,
            order_type: params.order_type,
        };

        Self::get_message_for_order(vec![order])
    }

    pub async fn market_open_with_builder(
        params: MarketOrderParams,
        builder: BuilderInfo,
    ) -> Result<Value> {
        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only: false,
            limit_px: params.px.parse::<f64>().unwrap(),
            sz: params.sz.parse::<f64>().unwrap(),
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Ioc".to_string(),
            }),
        };

        let value = Self::bulk_order_with_builder(vec![order], builder)?;
        Ok(value)
    }

    pub fn get_message_for_order(orders: Vec<ClientOrderRequest>) -> Result<MessageResponse> {
        let mut transformed_orders = Vec::new();

        for order in orders {
            transformed_orders.push(order.convert()?);
        }

        let bulk_order = BulkOrder {
            orders: transformed_orders,
            grouping: "na".to_string(),
            builder: None,
        };
        let action = Actions::Order(bulk_order.clone());

        Self::get_message_for_action(action, None)
    }

    pub fn bulk_order_with_builder(
        orders: Vec<ClientOrderRequest>,
        mut builder: BuilderInfo,
    ) -> Result<Value> {
        let mut transformed_orders = Vec::new();

        for order in orders {
            transformed_orders.push(order.convert()?);
        }

        builder.builder = builder.builder.to_lowercase();

        let action = Actions::Order(BulkOrder {
            orders: transformed_orders,
            grouping: "na".to_string(),
            builder: Some(builder),
        });
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
    }

    pub async fn cancel_order(cancel: ClientCancelRequest) -> Result<MessageResponse> {
        let mut transformed_cancels = Vec::new();

        transformed_cancels.push(CancelRequest {
            asset: cancel.asset,
            oid: cancel.oid,
        });

        let action = Actions::Cancel(BulkCancel {
            cancels: transformed_cancels,
        });

        Self::get_message_for_action(action, None)
    }

    pub async fn bulk_modify(modifies: Vec<ClientModifyRequest>) -> Result<Value> {
        let mut transformed_modifies = Vec::new();
        for modify in modifies.into_iter() {
            transformed_modifies.push(ModifyRequest {
                oid: modify.oid,
                order: modify.order.convert()?,
            });
        }

        let action = Actions::BatchModify(BulkModify {
            modifies: transformed_modifies,
        });

        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
    }

    pub async fn bulk_cancel_by_cloid(cancels: Vec<ClientCancelRequestCloid>) -> Result<Value> {
        let mut transformed_cancels: Vec<CancelRequestCloid> = Vec::new();
        for cancel in cancels.into_iter() {
            transformed_cancels.push(CancelRequestCloid {
                asset: cancel.asset,
                cloid: uuid_to_hex_string(cancel.cloid),
            });
        }

        let action = Actions::CancelByCloid(BulkCancelCloid {
            cancels: transformed_cancels,
        });

        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
    }

    pub async fn update_leverage(request: UpdateLeverage) -> Result<MessageResponse> {
        let action = Actions::UpdateLeverage(request);
        Self::get_message_for_action(action, None)
    }

    pub async fn spot_transfer(request: SpotTransferRequest) -> Result<MessageResponse> {
        let SpotTransferRequest {
            amount,
            destination,
            token,
        } = request;

        let timestamp = next_nonce();

        let spot_send = SpotSend {
            signature_chain_id: SIGNATURE_CHAIN_ID.into(),
            hyperliquid_chain: HYPERLIQUID_CHAIN.to_string(),
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: timestamp,
            token: token.to_string(),
        };
        let action = Actions::SpotSend(spot_send);

        Self::get_message_for_action(action, Some(timestamp))
    }

    pub async fn update_isolated_margin(
        amount: f64,
        asset: u32,
        is_buy: bool,
        nonce: u64,
    ) -> Result<Value> {
        // let amount = (amount * 1_000_000.0).round() as i64;

        let action = Actions::UpdateIsolatedMargin(UpdateIsolatedMargin {
            asset,
            is_buy,
            ntli: amount as i64,
        });
        let message = action.hash(nonce, None)?;
        let action = serde_json::to_value(&message).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
    }
    pub fn get_message_for_action(action: Actions, nonce: Option<u64>) -> Result<MessageResponse> {
        let nonce = nonce.unwrap_or(next_nonce());
        let connection_id = action.hash(nonce, None)?;
        let message: H256 = encode_l1_action(connection_id)?;

        Ok(MessageResponse {
            action,
            message,
            nonce,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        exchange::order::{Limit, OrderRequest},
        Order,
    };

    #[test]
    fn test_limit_order_action_hashing() -> Result<()> {
        let action = Actions::Order(BulkOrder {
            orders: vec![OrderRequest {
                asset: 3,
                is_buy: true,
                limit_px: "80000".to_string(),
                sz: "0.01".to_string(),
                reduce_only: false,
                order_type: Order::Limit(Limit {
                    tif: "Gtc".to_string(),
                }),
                cloid: None,
            }],
            grouping: "na".to_string(),
            builder: None,
        });
        let connection_id = action.hash(1745418656072, None)?;
        println!("connection_id: {}", connection_id);
        let message: H256 = encode_l1_action(connection_id)?;

        println!("message: {:?}", message);

        Ok(())
    }
}
