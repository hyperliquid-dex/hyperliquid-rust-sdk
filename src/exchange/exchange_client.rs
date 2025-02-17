use std::collections::HashMap;

use alloy::{
    primitives::{keccak256, Address, Signature, B256},
    signers::local::PrivateKeySigner,
};
use log::debug;
use reqwest::Client;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use crate::{
    exchange::{
        actions::{
            ApproveAgent, ApproveBuilderFee, BulkCancel, BulkModify, BulkOrder, ClaimRewards,
            EvmUserModify, ScheduleCancel, SendAsset, SetReferrer, UpdateIsolatedMargin,
            UpdateLeverage, UsdSend,
        },
        cancel::{CancelRequest, CancelRequestCloid, ClientCancelRequestCloid},
        modify::{ClientModifyRequest, ModifyRequest},
        order::{MarketCloseParams, MarketOrderParams},
        BuilderInfo, ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest,
    },
    helpers::{next_nonce, uuid_to_hex_string},
    info::info_client::InfoClient,
    meta::Meta,
    prelude::*,
    req::HttpClient,
    signature::{sign_l1_action, sign_typed_data},
    BaseUrl, BulkCancelCloid, ClassTransfer, Error, ExchangeResponseStatus, SpotSend, SpotUser,
    VaultTransfer, Withdraw3,
};

#[derive(Debug, Clone)]
pub struct ExchangeClient {
    pub http_client: HttpClient,
    pub wallet: PrivateKeySigner,
    pub meta: Meta,
    pub vault_address: Option<Address>,
    pub coin_to_asset: HashMap<String, u32>,
}

fn serialize_sig<S>(sig: &Signature, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = s.serialize_struct("Signature", 3)?;
    state.serialize_field("r", &sig.r())?;
    state.serialize_field("s", &sig.s())?;
    state.serialize_field("v", &(27 + sig.v() as u64))?;
    state.end()
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExchangePayload {
    action: serde_json::Value,
    #[serde(serialize_with = "serialize_sig")]
    signature: Signature,
    nonce: u64,
    vault_address: Option<Address>,
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
    SendAsset(SendAsset),
    VaultTransfer(VaultTransfer),
    SpotSend(SpotSend),
    SetReferrer(SetReferrer),
    ApproveBuilderFee(ApproveBuilderFee),
    EvmUserModify(EvmUserModify),
    ScheduleCancel(ScheduleCancel),
    ClaimRewards(ClaimRewards),
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

    pub async fn post(
        &self,
        action: serde_json::Value,
        signature: Signature,
        nonce: u64,
    ) -> Result<ExchangeResponseStatus> {
        // let signature = ExchangeSignature {
        //     r: signature.r(),
        //     s: signature.s(),
        //     v: 27 + signature.v() as u64,
        // };

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
        debug!("Response: {output}");
        serde_json::from_str(output).map_err(|e| Error::JsonParse(e.to_string()))
    }

    pub async fn enable_big_blocks(
        &self,
        using_big_blocks: bool,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);

        let timestamp = next_nonce();

        let action = Actions::EvmUserModify(EvmUserModify { using_big_blocks });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn usdc_transfer(
        &self,
        amount: &str,
        destination: &str,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let timestamp = next_nonce();
        let usd_send = UsdSend {
            signature_chain_id: 421614,
            hyperliquid_chain,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: timestamp,
        };
        let signature = sign_typed_data(&usd_send, wallet)?;
        let action = serde_json::to_value(Actions::UsdSend(usd_send))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp).await
    }

    pub async fn class_transfer(
        &self,
        usdc: f64,
        to_perp: bool,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        // payload expects usdc without decimals
        let usdc = (usdc * 1e6).round() as u64;
        let wallet = wallet.unwrap_or(&self.wallet);

        let timestamp = next_nonce();

        let action = Actions::SpotUser(SpotUser {
            class_transfer: ClassTransfer { usdc, to_perp },
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn send_asset(
        &self,
        destination: &str,
        source_dex: &str,
        destination_dex: &str,
        token: &str,
        amount: f64,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);

        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let timestamp = next_nonce();

        // Build fromSubAccount string (similar to Python SDK)
        let from_sub_account = self
            .vault_address
            .map_or_else(String::new, |vault_addr| format!("{vault_addr:?}"));

        let send_asset = SendAsset {
            signature_chain_id: 421614,
            hyperliquid_chain,
            destination: destination.to_string(),
            source_dex: source_dex.to_string(),
            destination_dex: destination_dex.to_string(),
            token: token.to_string(),
            amount: amount.to_string(),
            from_sub_account,
            nonce: timestamp,
        };

        let signature = sign_typed_data(&send_asset, wallet)?;
        let action = serde_json::to_value(Actions::SendAsset(send_asset))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp).await
    }

    pub async fn vault_transfer(
        &self,
        is_deposit: bool,
        usd: u64,
        vault_address: Option<Address>,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let vault_address = self
            .vault_address
            .or(vault_address)
            .ok_or(Error::VaultAddressNotFound)?;
        let wallet = wallet.unwrap_or(&self.wallet);

        let timestamp = next_nonce();

        let action = Actions::VaultTransfer(VaultTransfer {
            vault_address,
            is_deposit,
            usd,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

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
        let wallet = params.wallet.unwrap_or(&self.wallet);

        let base_url = match self.http_client.base_url.as_str() {
            "https://api.hyperliquid.xyz" => BaseUrl::Mainnet,
            "https://api.hyperliquid-testnet.xyz" => BaseUrl::Testnet,
            _ => return Err(Error::GenericRequest("Invalid base URL".to_string())),
        };
        let info_client = InfoClient::new(None, Some(base_url)).await?;
        let user_state = info_client.user_state(wallet.address()).await?;

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

        self.order(order, Some(wallet)).await
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
    ) -> Result<(B256, ExchangeResponseStatus)> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let agent = PrivateKeySigner::random();

        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let nonce = next_nonce();
        let approve_agent = ApproveAgent {
            signature_chain_id: 421614,
            hyperliquid_chain,
            agent_address: agent.address(),
            agent_name: None,
            nonce,
        };
        let signature = sign_typed_data(&approve_agent, wallet)?;
        let action = serde_json::to_value(Actions::ApproveAgent(approve_agent))
            .map_err(|e| Error::JsonParse(e.to_string()))?;
        Ok((agent.to_bytes(), self.post(action, signature, nonce).await?))
    }

    pub async fn withdraw_from_bridge(
        &self,
        amount: &str,
        destination: &str,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let timestamp = next_nonce();
        let withdraw = Withdraw3 {
            signature_chain_id: 421614,
            hyperliquid_chain,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: timestamp,
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
        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let timestamp = next_nonce();
        let spot_send = SpotSend {
            signature_chain_id: 421614,
            hyperliquid_chain,
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: timestamp,
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
        builder: Address,
        max_fee_rate: String,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = next_nonce();

        let hyperliquid_chain = if self.http_client.is_mainnet() {
            "Mainnet".to_string()
        } else {
            "Testnet".to_string()
        };

        let approve_builder_fee = ApproveBuilderFee {
            signature_chain_id: 421614,
            hyperliquid_chain,
            builder,
            max_fee_rate,
            nonce: timestamp,
        };
        let signature = sign_typed_data(&approve_builder_fee, wallet)?;
        let action = serde_json::to_value(Actions::ApproveBuilderFee(approve_builder_fee))
            .map_err(|e| Error::JsonParse(e.to_string()))?;

        self.post(action, signature, timestamp).await
    }

    pub async fn schedule_cancel(
        &self,
        time: Option<u64>,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = next_nonce();

        let action = Actions::ScheduleCancel(ScheduleCancel { time });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn claim_rewards(
        &self,
        wallet: Option<&PrivateKeySigner>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = next_nonce();

        let action = Actions::ClaimRewards(ClaimRewards {});
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.is_mainnet();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

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

    use alloy::primitives::address;

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
        assert_eq!(signature.to_string(), "0x77957e58e70f43b6b68581f2dc42011fc384538a2e5b7bf42d5b936f19fbb67360721a8598727230f67080efee48c812a6a4442013fd3b0eed509171bef9f23f1c");

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(signature.to_string(), "0xcd0925372ff1ed499e54883e9a6205ecfadec748f80ec463fe2f84f1209648776377961965cb7b12414186b1ea291e95fd512722427efcbcfb3b0b2bcd4d79d01c");

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
        assert_eq!(signature.to_string(), "0xd3e894092eb27098077145714630a77bbe3836120ee29df7d935d8510b03a08f456de5ec1be82aa65fc6ecda9ef928b0445e212517a98858cfaa251c4cd7552b1c");

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(signature.to_string(), "0x3768349dbb22a7fd770fc9fc50c7b5124a7da342ea579b309f58002ceae49b4357badc7909770919c45d850aabb08474ff2b7b3204ae5b66d9f7375582981f111c");

        Ok(())
    }

    #[test]
    fn test_tpsl_order_action_hashing() -> Result<()> {
        for (tpsl, mainnet_signature, testnet_signature) in [
            (
                "tp",
                "0xb91e5011dff15e4b4a40753730bda44972132e7b75641f3cac58b66159534a170d422ee1ac3c7a7a2e11e298108a2d6b8da8612caceaeeb3e571de3b2dfda9e41b",
                "0x6df38b609904d0d4439884756b8f366f22b3a081801dbdd23f279094a2299fac6424cb0cdc48c3706aeaa368f81959e91059205403d3afd23a55983f710aee871b"
            ),
            (
                "sl",
                "0x8456d2ace666fce1bee1084b00e9620fb20e810368841e9d4dd80eb29014611a0843416e51b1529c22dd2fc28f7ff8f6443875635c72011f60b62cbb8ce90e2d1c",
                "0xeb5bdb52297c1d19da45458758bd569dcb24c07e5c7bd52cf76600fd92fdd8213e661e21899c985421ec018a9ee7f3790e7b7d723a9932b7b5adcd7def5354601c"
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
        assert_eq!(signature.to_string(), "0x02f76cc5b16e0810152fa0e14e7b219f49c361e3325f771544c6f54e157bf9fa17ed0afc11a98596be85d5cd9f86600aad515337318f7ab346e5ccc1b03425d51b");

        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(signature.to_string(), "0x6ffebadfd48067663390962539fbde76cfa36f53be65abe2ab72c9db6d0db44457720db9d7c4860f142a484f070c84eb4b9694c3a617c83f0d698a27e55fd5e01c");

        Ok(())
    }

    #[test]
    fn test_approve_builder_fee_signing() -> Result<()> {
        let wallet = get_wallet()?;

        // Test mainnet
        let mainnet_fee = ApproveBuilderFee {
            signature_chain_id: 421614,
            hyperliquid_chain: "Mainnet".to_string(),
            builder: address!("0x1234567890123456789012345678901234567890"),
            max_fee_rate: "0.001%".to_string(),
            nonce: 1583838,
        };

        let mainnet_signature = sign_typed_data(&mainnet_fee, &wallet)?;
        assert_eq!(
            mainnet_signature.to_string(),
            "0x343c9078af7c3d6683abefd0ca3b2960de5b669b716863e6dc49090853a4a3cd6c016301239461091a8ca3ea5ac783362526c4d9e9e624ffc563aea93d6ac2391b"
        );

        // Test testnet
        let testnet_fee = ApproveBuilderFee {
            signature_chain_id: 421614,
            hyperliquid_chain: "Testnet".to_string(),
            builder: address!("0x1234567890123456789012345678901234567890"),
            max_fee_rate: "0.001%".to_string(),
            nonce: 1583838,
        };

        let testnet_signature = sign_typed_data(&testnet_fee, &wallet)?;
        assert_eq!(
            testnet_signature.to_string(),
            "0x2ada43eeebeba9cfe13faf95aa84e5b8c4885c3a07cbf4536f2df5edd340d4eb1ed0e24f60a80d199a842258d5fa737a18d486f7d4e656268b434d226f2811d71c"
        );

        // Verify signatures are different for mainnet vs testnet
        assert_ne!(mainnet_signature, testnet_signature);

        Ok(())
    }

    #[test]
    fn test_approve_builder_fee_hash() -> Result<()> {
        let action = Actions::ApproveBuilderFee(ApproveBuilderFee {
            signature_chain_id: 421614,
            hyperliquid_chain: "Mainnet".to_string(),
            builder: address!("0x1234567890123456789012345678901234567890"),
            max_fee_rate: "0.001%".to_string(),
            nonce: 1583838,
        });

        let connection_id = action.hash(1583838, None)?;
        assert_eq!(
            connection_id.to_string(),
            "0xbe889a23135fce39a37315424cc4ae910edea7b42a075457b15bf4a9f0a8cfa4"
        );

        Ok(())
    }

    #[test]
    fn test_claim_rewards_action_hashing() -> Result<()> {
        let wallet = get_wallet()?;
        let action = Actions::ClaimRewards(ClaimRewards {});
        let connection_id = action.hash(1583838, None)?;

        // Test mainnet signature
        let signature = sign_l1_action(&wallet, connection_id, true)?;
        assert_eq!(
            signature.to_string(),
            "0xe13542800ba5ec821153401e1cafac484d1f861adbbb86c00b580ec2560c153248b8d9f0e004ecc86959c07d44b591861ebab2167b54651a81367e2c3d472d4e1c"
        );

        // Test testnet signature
        let signature = sign_l1_action(&wallet, connection_id, false)?;
        assert_eq!(
            signature.to_string(),
            "0x16de9b346ddd8e200492a2db45ec9104dcdfc7fbfdbcd85890a6063bdd56df2c44846714c261a431de7095ad52e07143346eb26d9e66c6aed4674f120a1048131c"
        );

        Ok(())
    }

    #[test]
    fn test_send_asset_signing() -> Result<()> {
        let wallet = get_wallet()?;

        // Test mainnet - send asset to another address
        let mainnet_send = SendAsset {
            signature_chain_id: 421614,
            hyperliquid_chain: "Mainnet".to_string(),
            destination: "0x1234567890123456789012345678901234567890".to_string(),
            source_dex: "".to_string(),
            destination_dex: "spot".to_string(),
            token: "PURR:0xc4bf3f870c0e9465323c0b6ed28096c2".to_string(),
            amount: "100".to_string(),
            from_sub_account: "".to_string(),
            nonce: 1583838,
        };

        let mainnet_signature = sign_typed_data(&mainnet_send, &wallet)?;
        // Signature generated successfully - just verify it's a valid signature object

        // Test testnet - send different token
        let testnet_send = SendAsset {
            signature_chain_id: 421614,
            hyperliquid_chain: "Testnet".to_string(),
            destination: "0x1234567890123456789012345678901234567890".to_string(),
            source_dex: "spot".to_string(),
            destination_dex: "".to_string(),
            token: "USDC".to_string(),
            amount: "50".to_string(),
            from_sub_account: "".to_string(),
            nonce: 1583838,
        };

        let testnet_signature = sign_typed_data(&testnet_send, &wallet)?;
        // Verify signatures are different for mainnet vs testnet
        assert_ne!(mainnet_signature, testnet_signature);

        // Test with vault/subaccount
        let vault_send = SendAsset {
            signature_chain_id: 421614,
            hyperliquid_chain: "Mainnet".to_string(),
            destination: "0x1234567890123456789012345678901234567890".to_string(),
            source_dex: "".to_string(),
            destination_dex: "spot".to_string(),
            token: "PURR:0xc4bf3f870c0e9465323c0b6ed28096c2".to_string(),
            amount: "100".to_string(),
            from_sub_account: "0xabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd".to_string(),
            nonce: 1583838,
        };

        let vault_signature = sign_typed_data(&vault_send, &wallet)?;
        // Verify vault signature is different from non-vault signature
        assert_ne!(mainnet_signature, vault_signature);

        Ok(())
    }
}
