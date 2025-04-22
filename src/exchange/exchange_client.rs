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
    signature::encode_l1_action,
    BulkCancelCloid, Error,
};
use crate::{ClassTransfer, SpotSend, SpotUser, VaultTransfer, Withdraw3};
use ethers::types::{H160, H256};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use super::cancel::ClientCancelRequestCloid;
use super::order::{MarketCloseParams, MarketOrderParams};
use super::{BuilderInfo, ClientLimit, ClientOrder};

#[cfg(not(feature = "testnet"))]
const HYPERLIQUID_CHAIN: &str = "Mainnet";

#[cfg(feature = "testnet")]
const HYPERLIQUID_CHAIN: &str = "Testnet";

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
    SpotUser(SpotUser),
    VaultTransfer(VaultTransfer),
    SpotSend(SpotSend),
    SetReferrer(SetReferrer),
    ApproveBuilderFee(ApproveBuilderFee),
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct MessageResponse {
    pub action: BulkOrder,
    pub signature: String,
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

    pub async fn class_transfer(usdc: f64, to_perp: bool) -> Result<Value> {
        // payload expects usdc without decimals
        let usdc = (usdc * 1e6).round() as u64;

        let action = Actions::SpotUser(SpotUser {
            class_transfer: ClassTransfer { usdc, to_perp },
        });
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
    }

    pub async fn vault_transfer(
        is_deposit: bool,
        usd: String,
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
        let slippage = params.slippage.unwrap_or(0.05); // Default 5% slippage

        let px = Self::calculate_slippage_price(
            params.is_buy,
            slippage,
            params.px,
            params.price_decimals,
        )
        .await?;

        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only: false,
            limit_px: px,
            sz: params.sz,
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Ioc".to_string(),
            }),
        };

        Self::get_message_for_order(vec![order], params.nonce)
    }

    pub async fn market_open_with_builder(
        params: MarketOrderParams,
        builder: BuilderInfo,
    ) -> Result<Value> {
        let slippage = params.slippage.unwrap_or(0.05); // Default 5% slippage
        let px = Self::calculate_slippage_price(
            params.is_buy,
            slippage,
            params.px,
            params.price_decimals,
        )
        .await?;

        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only: false,
            limit_px: px,
            sz: params.sz,
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Ioc".to_string(),
            }),
        };

        let value = Self::bulk_order_with_builder(vec![order], builder)?;
        Ok(value)
    }

    pub async fn market_close(params: MarketCloseParams) -> Result<MessageResponse> {
        let slippage = params.slippage.unwrap_or(0.05); // Default 5% slippage

        let px = Self::calculate_slippage_price(
            params.is_buy,
            slippage,
            params.px,
            params.price_decimals,
        )
        .await?;

        let sz = round_to_decimals(params.sz, params.price_decimals);

        let order = ClientOrderRequest {
            asset: params.asset,
            is_buy: params.is_buy,
            reduce_only: true,
            limit_px: px,
            sz,
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Ioc".to_string(),
            }),
        };

        Self::get_message_for_order(vec![order], params.nonce)
    }

    pub fn get_message_for_order(
        orders: Vec<ClientOrderRequest>,
        nonce: u64,
    ) -> Result<MessageResponse> {
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

        let connection_id = action.hash(nonce, None)?;

        let signature: H256 = encode_l1_action(connection_id)?;

        Ok(MessageResponse {
            action: bulk_order,
            signature: hex::encode(signature),
        })
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

    pub async fn bulk_cancel(cancels: Vec<ClientCancelRequest>) -> Result<Value> {
        let mut transformed_cancels = Vec::new();
        for cancel in cancels.into_iter() {
            transformed_cancels.push(CancelRequest {
                asset: cancel.asset,
                oid: cancel.oid,
            });
        }

        let action = Actions::Cancel(BulkCancel {
            cancels: transformed_cancels,
        });

        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
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

    pub async fn update_leverage(leverage: u32, asset: u32, is_cross: bool) -> Result<Value> {
        let action = Actions::UpdateLeverage(UpdateLeverage {
            asset,
            is_cross,
            leverage,
        });
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
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

    pub async fn approve_builder_fee(builder: String, max_fee_rate: String) -> Result<H256> {
        let timestamp = next_nonce();

        let action = Actions::ApproveBuilderFee(ApproveBuilderFee {
            signature_chain_id: 421614.into(),
            hyperliquid_chain: HYPERLIQUID_CHAIN.to_string(),
            builder,
            max_fee_rate,
            nonce: timestamp,
        });
        let message = action.hash(timestamp, None)?;

        Ok(message)
    }

    async fn calculate_slippage_price(
        is_buy: bool,
        slippage: f64,
        px: f64,
        price_decimals: u32,
    ) -> Result<f64> {
        let slippage_factor = if is_buy {
            1.0 + slippage
        } else {
            1.0 - slippage
        };
        let px = px * slippage_factor;

        // Round to the correct number of decimal places and significant figures
        let px = round_to_significant_and_decimal(px, 5, price_decimals);

        Ok(px)
    }
}

fn round_to_decimals(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    (value * factor).round() / factor
}

fn round_to_significant_and_decimal(value: f64, sig_figs: u32, max_decimals: u32) -> f64 {
    let abs_value = value.abs();
    let magnitude = abs_value.log10().floor() as i32;
    let scale = 10f64.powi(sig_figs as i32 - magnitude - 1);
    let rounded = (abs_value * scale).round() / scale;
    round_to_decimals(rounded.copysign(value), max_decimals)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethers::signers::LocalWallet;

    use super::*;
    use crate::{
        exchange::order::{Limit, OrderRequest, Trigger},
        Order,
    };

    fn get_wallet() -> Result<LocalWallet> {
        let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
        priv_key
            .parse::<LocalWallet>()
            .map_err(|e| Error::Wallet(e.to_string()))
    }

    #[test]
    fn test_limit_order_action_hashing() -> Result<()> {
        let wallet = get_wallet()?;
        let action = Actions::Order(BulkOrder {
            orders: vec![OrderRequest {
                asset: 1,
                is_buy: true,
                limit_px: "2000.0".to_string(),
                sz: "3.5".to_string(),
                reduce_only: false,
                order_type: Order::Limit(Limit {
                    tif: "Ioc".to_string(),
                }),
                cloid: None,
            }],
            grouping: "na".to_string(),
            builder: None,
        });
        let connection_id = action.hash(1583838, None)?;

        Ok(())
    }
}
