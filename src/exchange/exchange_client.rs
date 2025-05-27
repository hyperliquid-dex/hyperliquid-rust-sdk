use crate::signature::sign_typed_data;
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
    info::info_client::InfoClient,
    meta::Meta,
    prelude::*,
    req::HttpClient,
    signature::sign_l1_action,
    BaseUrl, BulkCancelCloid, Error, ExchangeResponseStatus,
};
use crate::{ClassTransfer, SpotSend, SpotUser, VaultTransfer, Withdraw3};

use alloy::primitives::{keccak256, B256, U256};
use alloy::signers::k256::ecdsa::SigningKey;
use alloy::signers::Signature;
use alloy::{primitives::Address, signers::local::PrivateKeySigner};

use log::debug;
use reqwest::Client;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::cancel::ClientCancelRequestCloid;
use super::order::{MarketCloseParams, MarketOrderParams};
use super::{BuilderInfo, ClientLimit, ClientOrder};

#[derive(Debug)]
pub struct ExchangeClient {
    pub http_client: HttpClient,
    pub wallet: PrivateKeySigner,
    pub meta: Meta,
    pub vault_address: Option<Address>,
    pub coin_to_asset: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExchangePayload {
    action: serde_json::Value,
    #[serde(serialize_with = "serialize_signature_legacy")]
    signature: Signature,
    nonce: u64,
    vault_address: Option<Address>,
}

pub fn serialize_signature_legacy<S>(
    signature: &Signature,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut s = serializer.serialize_struct("Signature", 4)?;
    s.serialize_field("r", &signature.r())?;
    s.serialize_field("s", &signature.s())?;
    s.serialize_field("y_parity", &signature.v())?;
    s.serialize_field("v", &(27 + signature.v() as u8))?;
    s.end()
}

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
    fn hash(&self, timestamp: u64, vault_address: Option<Address>) -> Result<B256> {
        let mut bytes =
            rmp_serde::to_vec_named(self).map_err(|e| Error::RmpParse(e.to_string()))?;
        bytes.extend(timestamp.to_be_bytes());
        if let Some(vault_address) = vault_address {
            bytes.push(1);
            bytes.extend(vault_address);
        } else {
            bytes.push(0);
        }
        Ok(keccak256(bytes))
    }
}

impl ExchangeClient {
    pub async fn new(
        client: Option<Client>,
        wallet: PrivateKeySigner,
        base_url: Option<BaseUrl>,
        meta: Option<Meta>,
        vault_address: Option<Address>,
    ) -> Result<ExchangeClient> {
        let client = client.unwrap_or_default();
        let base_url = base_url.unwrap_or(BaseUrl::Mainnet);

        let info = InfoClient::new(None, Some(base_url)).await?;
        let meta = if let Some(meta) = meta {
            meta
        } else {
            info.meta().await?
        };

        let mut coin_to_asset = HashMap::new();
        for (asset_ind, asset) in meta.universe.iter().enumerate() {
            coin_to_asset.insert(asset.name.clone(), asset_ind as u32);
        }

        coin_to_asset = info
            .spot_meta()
            .await?
            .add_pair_and_name_to_index_map(coin_to_asset);

        Ok(ExchangeClient {
            wallet,
            meta,
            vault_address,
            http_client: HttpClient {
                client,
                base_url: base_url.get_url(),
            },
            coin_to_asset,
        })
    }

    async fn post(
        &self,
        action: serde_json::Value,
        signature: Signature,
        nonce: u64,
    ) -> Result<ExchangeResponseStatus> {
        let exchange_payload = ExchangePayload {
            action,
            signature,
            nonce,
            vault_address: self.vault_address,
        };

        let res = serde_json::to_string(&exchange_payload)
            .map_err(|e| Error::JsonParse(e.to_string()))?;
        debug!("Sending request {res:?}");

        let output = &self
            .http_client
            .post("/exchange", res)
            .await
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        serde_json::from_str(output).map_err(|e| Error::JsonParse(e.to_string()))
    }

    pub async fn usdc_transfer(
        &self,
        amount: &str,
        destination: &str,
        signer: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let signer = signer.unwrap_or(&self.wallet);
        let hyperliquid_chain = self.http_client.get_network();

        let timestamp = next_nonce();
        let usd_send = UsdSend {
            signature_chain_id: U256::from(421614),
            hyperliquid_chain,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: U256::from(timestamp),
        };
        let signature = sign_typed_data(&usd_send, signer)?;
        let action = serde_json::to_value(Actions::UsdSend(usd_send))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp).await
    }

    pub async fn class_transfer(
        &self,
        usdc: f64,
        to_perp: bool,
        signer: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        // payload expects usdc without decimals
        let usdc = (usdc * 1e6).round() as u64;
        let signer = signer.unwrap_or(&self.wallet);

        let timestamp = next_nonce();

        let action = Actions::SpotUser(SpotUser {
            class_transfer: ClassTransfer { usdc, to_perp },
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(signer, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn vault_transfer(
        &self,
        is_deposit: bool,
        usd: u64,
        vault_address: Option<Address>,
        signer: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let vault_address = self
            .vault_address
            .or(vault_address)
            .ok_or(Error::VaultAddressNotFound)?;
        let signer = signer.unwrap_or(&self.wallet);

        let timestamp = next_nonce();

        let action = Actions::VaultTransfer(VaultTransfer {
            vault_address,
            is_deposit,
            usd,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(signer, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn market_open(
        &self,
        params: MarketOrderParams<'_>,
    ) -> Result<ExchangeResponseStatus> {
        let slippage = params.slippage.unwrap_or(0.05); // Default 5% slippage
        let (px, sz_decimals) = self
            .calculate_slippage_price(params.asset, params.is_buy, slippage, params.px)
            .await?;

        let order = ClientOrderRequest {
            asset: params.asset.to_string(),
            is_buy: params.is_buy,
            reduce_only: false,
            limit_px: px,
            sz: round_to_decimals(params.sz, sz_decimals),
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Ioc".to_string(),
            }),
        };

        self.order(order, params.wallet).await
    }

    pub async fn market_open_with_builder(
        &self,
        params: MarketOrderParams<'_>,
        builder: BuilderInfo,
    ) -> Result<ExchangeResponseStatus> {
        let slippage = params.slippage.unwrap_or(0.05); // Default 5% slippage
        let (px, sz_decimals) = self
            .calculate_slippage_price(params.asset, params.is_buy, slippage, params.px)
            .await?;

        let order = ClientOrderRequest {
            asset: params.asset.to_string(),
            is_buy: params.is_buy,
            reduce_only: false,
            limit_px: px,
            sz: round_to_decimals(params.sz, sz_decimals),
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Ioc".to_string(),
            }),
        };

        self.order_with_builder(order, params.wallet, builder).await
    }

    pub async fn market_close(
        &self,
        params: MarketCloseParams<'_>,
    ) -> Result<ExchangeResponseStatus> {
        let slippage = params.slippage.unwrap_or(0.05); // Default 5% slippage
        let signer = params.wallet.unwrap_or(&self.wallet);

        let base_url = match self.http_client.base_url.as_str() {
            "https://api.hyperliquid.xyz" => BaseUrl::Mainnet,
            "https://api.hyperliquid-testnet.xyz" => BaseUrl::Testnet,
            _ => return Err(Error::GenericRequest("Invalid base URL".to_string())),
        };
        let info_client = InfoClient::new(None, Some(base_url)).await?;
        let user_state = info_client.user_state(signer.address()).await?;

        let position = user_state
            .asset_positions
            .iter()
            .find(|p| p.position.coin == params.asset)
            .ok_or(Error::AssetNotFound)?;

        let szi = position
            .position
            .szi
            .parse::<f64>()
            .map_err(|_| Error::FloatStringParse)?;

        let (px, sz_decimals) = self
            .calculate_slippage_price(params.asset, szi < 0.0, slippage, params.px)
            .await?;

        let sz = round_to_decimals(params.sz.unwrap_or_else(|| szi.abs()), sz_decimals);

        let order = ClientOrderRequest {
            asset: params.asset.to_string(),
            is_buy: szi < 0.0,
            reduce_only: true,
            limit_px: px,
            sz,
            cloid: params.cloid,
            order_type: ClientOrder::Limit(ClientLimit {
                tif: "Ioc".to_string(),
            }),
        };

        self.order(order, Some(signer)).await
    }

    async fn calculate_slippage_price(
        &self,
        asset: &str,
        is_buy: bool,
        slippage: f64,
        px: Option<f64>,
    ) -> Result<(f64, u32)> {
        let base_url = match self.http_client.base_url.as_str() {
            "https://api.hyperliquid.xyz" => BaseUrl::Mainnet,
            "https://api.hyperliquid-testnet.xyz" => BaseUrl::Testnet,
            _ => return Err(Error::GenericRequest("Invalid base URL".to_string())),
        };
        let info_client = InfoClient::new(None, Some(base_url)).await?;
        let meta = info_client.meta().await?;

        let asset_meta = meta
            .universe
            .iter()
            .find(|a| a.name == asset)
            .ok_or(Error::AssetNotFound)?;

        let sz_decimals = asset_meta.sz_decimals;
        let max_decimals: u32 = if self.coin_to_asset[asset] < 10000 {
            6
        } else {
            8
        };
        let price_decimals = max_decimals.saturating_sub(sz_decimals);

        let px = if let Some(px) = px {
            px
        } else {
            let all_mids = info_client.all_mids().await?;
            all_mids
                .get(asset)
                .ok_or(Error::AssetNotFound)?
                .parse::<f64>()
                .map_err(|_| Error::FloatStringParse)?
        };

        debug!("px before slippage: {px:?}");
        let slippage_factor = if is_buy {
            1.0 + slippage
        } else {
            1.0 - slippage
        };
        let px = px * slippage_factor;

        // Round to the correct number of decimal places and significant figures
        let px = round_to_significant_and_decimal(px, 5, price_decimals);

        debug!("px after slippage: {px:?}");
        Ok((px, sz_decimals))
    }

    pub async fn order(
        &self,
        order: ClientOrderRequest,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_order(vec![order], wallet).await
    }

    pub async fn order_with_builder(
        &self,
        order: ClientOrderRequest,
        wallet: Option<&PrivateKeySigner>,
        builder: BuilderInfo,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_order_with_builder(vec![order], wallet, builder)
            .await
    }

    pub async fn bulk_order(
        &self,
        orders: Vec<ClientOrderRequest>,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = next_nonce();

        let mut transformed_orders = Vec::new();

        for order in orders {
            transformed_orders.push(order.convert(&self.coin_to_asset)?);
        }

        let action = Actions::Order(BulkOrder {
            orders: transformed_orders,
            grouping: "na".to_string(),
            builder: None,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp).await
    }

    pub async fn bulk_order_with_builder(
        &self,
        orders: Vec<ClientOrderRequest>,
        wallet: Option<&PrivateKeySigner>,
        mut builder: BuilderInfo,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = next_nonce();

        builder.builder = builder.builder.to_lowercase();

        let mut transformed_orders = Vec::new();

        for order in orders {
            transformed_orders.push(order.convert(&self.coin_to_asset)?);
        }

        let action = Actions::Order(BulkOrder {
            orders: transformed_orders,
            grouping: "na".to_string(),
            builder: Some(builder),
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp).await
    }

    pub async fn cancel(
        &self,
        cancel: ClientCancelRequest,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_cancel(vec![cancel], wallet).await
    }

    pub async fn bulk_cancel(
        &self,
        cancels: Vec<ClientCancelRequest>,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = next_nonce();

        let mut transformed_cancels = Vec::new();
        for cancel in cancels.into_iter() {
            let &asset = self
                .coin_to_asset
                .get(&cancel.asset)
                .ok_or(Error::AssetNotFound)?;
            transformed_cancels.push(CancelRequest {
                asset,
                oid: cancel.oid,
            });
        }

        let action = Actions::Cancel(BulkCancel {
            cancels: transformed_cancels,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;

        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn modify(
        &self,
        modify: ClientModifyRequest,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_modify(vec![modify], wallet).await
    }

    pub async fn bulk_modify(
        &self,
        modifies: Vec<ClientModifyRequest>,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = next_nonce();

        let mut transformed_modifies = Vec::new();
        for modify in modifies.into_iter() {
            transformed_modifies.push(ModifyRequest {
                oid: modify.oid,
                order: modify.order.convert(&self.coin_to_asset)?,
            });
        }

        let action = Actions::BatchModify(BulkModify {
            modifies: transformed_modifies,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;

        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn cancel_by_cloid(
        &self,
        cancel: ClientCancelRequestCloid,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_cancel_by_cloid(vec![cancel], wallet).await
    }

    pub async fn bulk_cancel_by_cloid(
        &self,
        cancels: Vec<ClientCancelRequestCloid>,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = next_nonce();

        let mut transformed_cancels: Vec<CancelRequestCloid> = Vec::new();
        for cancel in cancels.into_iter() {
            let &asset = self
                .coin_to_asset
                .get(&cancel.asset)
                .ok_or(Error::AssetNotFound)?;
            transformed_cancels.push(CancelRequestCloid {
                asset,
                cloid: uuid_to_hex_string(cancel.cloid),
            });
        }

        let action = Actions::CancelByCloid(BulkCancelCloid {
            cancels: transformed_cancels,
        });

        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn update_leverage(
        &self,
        leverage: u32,
        coin: &str,
        is_cross: bool,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);

        let timestamp = next_nonce();

        let &asset_index = self.coin_to_asset.get(coin).ok_or(Error::AssetNotFound)?;
        let action = Actions::UpdateLeverage(UpdateLeverage {
            asset: asset_index,
            is_cross,
            leverage,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn update_isolated_margin(
        &self,
        amount: f64,
        coin: &str,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);

        let amount = (amount * 1_000_000.0).round() as i64;
        let timestamp = next_nonce();

        let &asset_index = self.coin_to_asset.get(coin).ok_or(Error::AssetNotFound)?;
        let action = Actions::UpdateIsolatedMargin(UpdateIsolatedMargin {
            asset: asset_index,
            is_buy: true,
            ntli: amount,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn approve_agent(
        &self,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<(SigningKey, ExchangeResponseStatus)> {
        let wallet = wallet.unwrap_or(&self.wallet);

        let random_signer = PrivateKeySigner::random();
        let key = random_signer.credential().clone();
        let address = random_signer.address();

        let hyperliquid_chain = self.http_client.get_network();

        let nonce = next_nonce();
        let approve_agent = ApproveAgent {
            signature_chain_id: U256::from(421614),
            hyperliquid_chain,
            agent_address: address,
            agent_name: Default::default(),
            nonce: U256::from(nonce),
        };
        let signature = sign_typed_data(&approve_agent, wallet)?;
        let action = serde_json::to_value(Actions::ApproveAgent(approve_agent))
            .map_err(|e| Error::JsonParse(e.to_string()))?;
        Ok((key, self.post(action, signature, nonce).await?))
    }

    pub async fn withdraw_from_bridge(
        &self,
        amount: &str,
        destination: &str,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let hyperliquid_chain = self.http_client.get_network();

        let timestamp = next_nonce();
        let withdraw = Withdraw3 {
            signature_chain_id: U256::from(421614),
            hyperliquid_chain,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: U256::from(timestamp),
        };
        let signature = sign_typed_data(&withdraw, wallet)?;
        let action = serde_json::to_value(Actions::Withdraw3(withdraw))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp).await
    }

    pub async fn spot_transfer(
        &self,
        amount: &str,
        destination: &str,
        token: &str,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let hyperliquid_chain = self.http_client.get_network();

        let timestamp = next_nonce();
        let spot_send = SpotSend {
            signature_chain_id: U256::from(421614),
            hyperliquid_chain,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: U256::from(timestamp),
            token: token.to_string(),
        };
        let signature = sign_typed_data(&spot_send, wallet)?;
        let action = serde_json::to_value(Actions::SpotSend(spot_send))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp).await
    }

    pub async fn set_referrer(
        &self,
        code: String,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = next_nonce();

        let action = Actions::SetReferrer(SetReferrer { code });

        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp).await
    }

    pub async fn approve_builder_fee(
        &self,
        builder: String,
        max_fee_rate: String,
        signer: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let signer = signer.unwrap_or(&self.wallet);
        let timestamp = next_nonce();

        let hyperliquid_chain = self.http_client.get_network();

        let action = Actions::ApproveBuilderFee(ApproveBuilderFee {
            signature_chain_id: U256::from(421614),
            hyperliquid_chain,
            builder,
            max_fee_rate,
            nonce: timestamp,
        });

        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(signer, connection_id, is_mainnet)?;
        self.post(action, signature, timestamp).await
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

    use super::*;
    use crate::{
        exchange::order::{Limit, OrderRequest, Trigger},
        Order,
    };

    fn get_wallet() -> Result<PrivateKeySigner> {
        let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
        priv_key
            .parse::<PrivateKeySigner>()
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
        assert_eq!(signature.to_string(), "0x82f7e6747e4fcd0359efa9490426871693472d07ce404261f1c39084beb7aba02d8e9e3f618336c2287849b69e5021ac593bb94c00ae82815c7580a4256923a01c");

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(signature.to_string(), "0x1760e47c9670cbc26ca6ad961818231fabcffb116e43feed75baa87c0307cc7c446131e4bd121caeba7fe1e8494410dbf149206988985bb66044f905450638821b");

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
        assert_eq!(signature.to_string(), "0xc5cc6ca48c2c4223c89f62f1e6eff4c68546dfc7baa12073a8ddff5a38b3e62a6d2967e080698522863ca147685e5c68ff854348dd97cc032771ff5be301a2c21b");

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(signature.to_string(), "0xeb99e4496d3897aa58c653044c543d347458edfc1a68182fd53c64bcf4c3a6e2429f53e4dee68214f32d3277f7454b77be963d3cb98b8462c79b47adf61185861b");

        Ok(())
    }

    #[test]
    fn test_tpsl_order_action_hashing() -> Result<()> {
        for (tpsl, mainnet_signature, testnet_signature) in [
            (
                "tp",
                "0x5061414c399533a5880f429362ee15511864401aefe00f8a1b6da937a0ecc2e058f90b1b72b26bb79d537785e49b37420b3a04d313ff7010ea8453bbf6e2383c1b",
                "0x09c29abc493d6144f1136f197194d0cdd87cd8c56c971ee38a69447da5d7a11773355e2d66afff017386e3143654dd07be365530ad382a246f14f818d66be5c81c"
            ),
            (
                "sl",
                "0x40fdee49426becc5cabebbcd61182cee72a0ae2c40df3de0cbbcf65621cf64b54dd9d720192c16dedf5aea10aca7bd5522ac76164a3a3f7d10b2054ff732a91e1b",
                "0x7f2cfdcaa1d8a0b47e4da1699f8aa4af8da749db6bd5ea29f18fdb880856b1705d52ec14e7fc1c37a728b3e6099c83a58c23d5d0775b800c175bf137e29dc0e91c"
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
        assert_eq!(signature.to_string(), "0x9f8b8530274f2f174adf8cd0f02e8bbf5c2987866fbac000e7e3e19214686dde1018d9d181e95a84246a7361ca1dd731a8cbd42dfb04cfc9b071e78aa487a6441c");

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(signature.to_string(), "0x6b50910d58758f2a50f9629ac8f01c2ce533d9f5db21446c8f9d3a720e0e5f5e7a2bcf0de855c9af0ddb4959575add4478f5eb153eee8d35b640238ed71314381b");

        Ok(())
    }
}
