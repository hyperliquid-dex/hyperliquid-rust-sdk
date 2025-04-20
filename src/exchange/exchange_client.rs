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
    BulkCancelCloid, Error,
};
use crate::{ClassTransfer, SpotSend, SpotUser, VaultTransfer, Withdraw3};
use ethers::types::{H160, H256};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::cancel::ClientCancelRequestCloid;
use super::order::{MarketCloseParams, MarketOrderParams};
use super::{BuilderInfo, ClientLimit, ClientOrder};

#[cfg(feature = "mainnet")]
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

    pub async fn market_open(params: MarketOrderParams) -> Result<Value> {
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

        Self::get_message_for_order(vec![order])
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

    pub async fn market_close(params: MarketCloseParams) -> Result<Value> {
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

        Self::get_message_for_order(vec![order])
    }

    pub fn get_message_for_order(orders: Vec<ClientOrderRequest>) -> Result<Value> {
        let mut transformed_orders = Vec::new();

        for order in orders {
            transformed_orders.push(order.convert()?);
        }

        let action = Actions::Order(BulkOrder {
            orders: transformed_orders,
            grouping: "na".to_string(),
            builder: None,
        });
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
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

    pub async fn update_isolated_margin(amount: f64, asset: u32, is_buy: bool) -> Result<Value> {
        // let amount = (amount * 1_000_000.0).round() as i64;

        let action = Actions::UpdateIsolatedMargin(UpdateIsolatedMargin {
            asset,
            is_buy,
            ntli: amount as i64,
        });
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
    }

    pub async fn approve_builder_fee(builder: String, max_fee_rate: String) -> Result<Value> {
        let timestamp = next_nonce();

        let action = Actions::ApproveBuilderFee(ApproveBuilderFee {
            signature_chain_id: 421614.into(),
            hyperliquid_chain: HYPERLIQUID_CHAIN.to_string(),
            builder,
            max_fee_rate,
            nonce: timestamp,
        });

        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        Ok(action)
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
        signature::sign_l1_action,
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

        let signature = sign_l1_action(&wallet, connection_id, true)?;
        assert_eq!(signature.to_string(), "77957e58e70f43b6b68581f2dc42011fc384538a2e5b7bf42d5b936f19fbb67360721a8598727230f67080efee48c812a6a4442013fd3b0eed509171bef9f23f1c");

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(signature.to_string(), "cd0925372ff1ed499e54883e9a6205ecfadec748f80ec463fe2f84f1209648776377961965cb7b12414186b1ea291e95fd512722427efcbcfb3b0b2bcd4d79d01c");

        Ok(())
    }

    #[test]
    fn test_limit_order_action_hashing_with_cloid() -> Result<()> {
        let cloid = uuid::Uuid::from_str("1e60610f-0b3d-4205-97c8-8c1fed2ad5ee")
            .map_err(|_e| uuid::Uuid::new_v4());
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
                cloid: Some(uuid_to_hex_string(cloid.unwrap())),
            }],
            grouping: "na".to_string(),
            builder: None,
        });
        let connection_id = action.hash(1583838, None)?;

        let signature = sign_l1_action(&wallet, connection_id, true)?;
        assert_eq!(signature.to_string(), "d3e894092eb27098077145714630a77bbe3836120ee29df7d935d8510b03a08f456de5ec1be82aa65fc6ecda9ef928b0445e212517a98858cfaa251c4cd7552b1c");

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(signature.to_string(), "3768349dbb22a7fd770fc9fc50c7b5124a7da342ea579b309f58002ceae49b4357badc7909770919c45d850aabb08474ff2b7b3204ae5b66d9f7375582981f111c");

        Ok(())
    }

    #[test]
    fn test_tpsl_order_action_hashing() -> Result<()> {
        for (tpsl, mainnet_signature, testnet_signature) in [
            (
                "tp",
                "b91e5011dff15e4b4a40753730bda44972132e7b75641f3cac58b66159534a170d422ee1ac3c7a7a2e11e298108a2d6b8da8612caceaeeb3e571de3b2dfda9e41b",
                "6df38b609904d0d4439884756b8f366f22b3a081801dbdd23f279094a2299fac6424cb0cdc48c3706aeaa368f81959e91059205403d3afd23a55983f710aee871b"
            ),
            (
                "sl",
                "8456d2ace666fce1bee1084b00e9620fb20e810368841e9d4dd80eb29014611a0843416e51b1529c22dd2fc28f7ff8f6443875635c72011f60b62cbb8ce90e2d1c",
                "eb5bdb52297c1d19da45458758bd569dcb24c07e5c7bd52cf76600fd92fdd8213e661e21899c985421ec018a9ee7f3790e7b7d723a9932b7b5adcd7def5354601c"
            )
        ] {
            let wallet = get_wallet()?;
            let action = Actions::Order(BulkOrder {
                orders: vec![
                    OrderRequest {
                        asset: 1,
                        is_buy: true,
                        limit_px: "2000.0".to_string(),
                        sz: "3.5".to_string(),
                        reduce_only: false,
                        order_type: Order::Trigger(Trigger {
                            trigger_px: "2000.0".to_string(),
                            is_market: true,
                            tpsl: tpsl.to_string(),
                        }),
                        cloid: None,
                    }
                ],
                grouping: "na".to_string(),
                builder: None,
            });
            let connection_id = action.hash(1583838, None)?;

            let signature = sign_l1_action(&wallet, connection_id, true)?;
            assert_eq!(signature.to_string(), mainnet_signature);

            let signature = sign_l1_action(&wallet, connection_id, false)?;
            assert_eq!(signature.to_string(), testnet_signature);
        }
        Ok(())
    }

    #[test]
    fn test_cancel_action_hashing() -> Result<()> {
        let wallet = get_wallet()?;
        let action = Actions::Cancel(BulkCancel {
            cancels: vec![CancelRequest {
                asset: 1,
                oid: 82382,
            }],
        });
        let connection_id = action.hash(1583838, None)?;

        let signature = sign_l1_action(&wallet, connection_id, true)?;
        assert_eq!(signature.to_string(), "02f76cc5b16e0810152fa0e14e7b219f49c361e3325f771544c6f54e157bf9fa17ed0afc11a98596be85d5cd9f86600aad515337318f7ab346e5ccc1b03425d51b");

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(signature.to_string(), "6ffebadfd48067663390962539fbde76cfa36f53be65abe2ab72c9db6d0db44457720db9d7c4860f142a484f070c84eb4b9694c3a617c83f0d698a27e55fd5e01c");

        Ok(())
    }
}
