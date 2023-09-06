use crate::{
    consts::MAINNET_API_URL,
    exchange::{
        actions::{
            AgentConnect, BulkCancel, BulkOrder, UpdateIsolatedMargin, UpdateLeverage, UsdcTransfer,
        },
        cancel::CancelRequest,
        ClientCancelRequest, ClientOrderRequest,
    },
    helpers::{generate_random_key, now_timestamp_ms, EthChain},
    info::info_client::InfoClient,
    meta::Meta,
    prelude::*,
    req::HttpClient,
    signature::{
        agent::mainnet::Agent, keccak, sign_l1_action, sign_usd_transfer_action, sign_with_agent,
        usdc_transfer::mainnet::UsdTransferSignPayload,
    },
    BaseUrl, Error, ExchangeResponseStatus,
};
use ethers::{
    abi::AbiEncode,
    signers::{LocalWallet, Signer},
    types::{Signature, H160, H256},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct ExchangeClient {
    pub http_client: HttpClient,
    pub wallet: LocalWallet,
    pub meta: Meta,
    pub vault_address: Option<H160>,
    pub coin_to_asset: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExchangePayload {
    action: serde_json::Value,
    signature: Signature,
    nonce: u64,
    vault_address: Option<H160>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Actions {
    UsdTransfer(UsdcTransfer),
    UpdateLeverage(UpdateLeverage),
    UpdateIsolatedMargin(UpdateIsolatedMargin),
    Order(BulkOrder),
    Cancel(BulkCancel),
    Connect(AgentConnect),
}

impl ExchangeClient {
    pub async fn new(
        client: Option<Client>,
        wallet: LocalWallet,
        base_url: Option<BaseUrl>,
        meta: Option<Meta>,
        vault_address: Option<H160>,
    ) -> Result<ExchangeClient> {
        let client = client.unwrap_or_else(Client::new);
        let base_url = base_url.unwrap_or(BaseUrl::Mainnet);

        let meta = if let Some(meta) = meta {
            meta
        } else {
            let info = InfoClient::new(None, Some(base_url)).await?;
            info.meta().await?
        };

        let mut coin_to_asset = HashMap::new();
        for (asset_ind, asset) in meta.universe.iter().enumerate() {
            coin_to_asset.insert(asset.name.clone(), asset_ind as u32);
        }

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

        serde_json::from_str(
            &self
                .http_client
                .post("/exchange", res)
                .await
                .map_err(|e| Error::JsonParse(e.to_string()))?,
        )
        .map_err(|e| Error::JsonParse(e.to_string()))
    }

    pub async fn usdc_transfer(
        &self,
        amount: &str,
        destination: &str,
        wallet: Option<&LocalWallet>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let (chain, l1_name) = if self.http_client.base_url.eq(MAINNET_API_URL) {
            (EthChain::Arbitrum, "Arbitrum".to_string())
        } else {
            (EthChain::ArbitrumGoerli, "ArbitrumGoerli".to_string())
        };

        let timestamp = now_timestamp_ms();
        let payload = serde_json::to_value(UsdTransferSignPayload {
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: timestamp,
        })
        .map_err(|e| Error::JsonParse(e.to_string()))?;
        let action = serde_json::to_value(Actions::UsdTransfer(UsdcTransfer {
            chain: l1_name,
            payload,
        }))
        .map_err(|e| Error::JsonParse(e.to_string()))?;

        let signature = sign_usd_transfer_action(wallet, chain, amount, destination, timestamp)?;
        self.post(action, signature, timestamp).await
    }

    pub async fn order(
        &self,
        order: ClientOrderRequest,
        wallet: Option<&LocalWallet>,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_order(vec![order], wallet).await
    }

    pub async fn bulk_order(
        &self,
        orders: Vec<ClientOrderRequest>,
        wallet: Option<&LocalWallet>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = now_timestamp_ms();
        let vault_address = self.vault_address.unwrap_or_default();

        let mut hashable_tuples = Vec::new();
        let mut transformed_orders = Vec::new();

        for order in orders {
            hashable_tuples.push(order.create_hashable_tuple(&self.coin_to_asset)?);
            transformed_orders.push(order.convert(&self.coin_to_asset)?);
        }

        let connection_id = keccak((hashable_tuples, 0, vault_address, timestamp));
        let action = serde_json::to_value(Actions::Order(BulkOrder {
            grouping: "na".to_string(),
            orders: transformed_orders,
        }))
        .map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.base_url == BaseUrl::Mainnet.get_url();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn cancel(
        &self,
        cancel: ClientCancelRequest,
        wallet: Option<&LocalWallet>,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_cancel(vec![cancel], wallet).await
    }

    pub async fn bulk_cancel(
        &self,
        cancels: Vec<ClientCancelRequest>,
        wallet: Option<&LocalWallet>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let timestamp = now_timestamp_ms();
        let vault_address = self.vault_address.unwrap_or_default();

        let mut hashable_tuples = Vec::new();
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
            hashable_tuples.push((asset, cancel.oid));
        }

        let connection_id = keccak((hashable_tuples, vault_address, timestamp));
        let action = serde_json::to_value(Actions::Cancel(BulkCancel {
            cancels: transformed_cancels,
        }))
        .map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.base_url == BaseUrl::Mainnet.get_url();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn update_leverage(
        &self,
        leverage: u32,
        coin: &str,
        is_cross: bool,
        wallet: Option<&LocalWallet>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);

        let timestamp = now_timestamp_ms();
        let vault_address = self.vault_address.unwrap_or_default();

        let &asset_index = self.coin_to_asset.get(coin).ok_or(Error::AssetNotFound)?;
        let connection_id = keccak((asset_index, is_cross, leverage, vault_address, timestamp));
        let action = serde_json::to_value(Actions::UpdateLeverage(UpdateLeverage {
            asset: asset_index,
            is_cross,
            leverage,
        }))
        .map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.base_url == BaseUrl::Mainnet.get_url();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn update_isolated_margin(
        &self,
        amount: f64,
        coin: &str,
        wallet: Option<&LocalWallet>,
    ) -> Result<ExchangeResponseStatus> {
        let wallet = wallet.unwrap_or(&self.wallet);

        let amount = (amount * 1_000_000.0).round() as i64;
        let timestamp = now_timestamp_ms();
        let vault_address = self.vault_address.unwrap_or_default();

        let &asset_index = self.coin_to_asset.get(coin).ok_or(Error::AssetNotFound)?;
        let connection_id = keccak((asset_index, true, amount, vault_address, timestamp));
        let action = serde_json::to_value(Actions::UpdateIsolatedMargin(UpdateIsolatedMargin {
            asset: asset_index,
            is_buy: true,
            ntli: amount,
        }))
        .map_err(|e| Error::JsonParse(e.to_string()))?;
        let is_mainnet = self.http_client.base_url == BaseUrl::Mainnet.get_url();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn approve_agent(
        &self,
        wallet: Option<&LocalWallet>,
    ) -> Result<(String, ExchangeResponseStatus)> {
        let wallet = wallet.unwrap_or(&self.wallet);
        let key = H256::from(generate_random_key()?).encode_hex()[2..].to_string();

        let address = key
            .parse::<LocalWallet>()
            .map_err(|e| Error::PrivateKeyParse(e.to_string()))?
            .address();
        let connection_id = keccak(address);

        let (chain, l1_name) = if self.http_client.base_url.eq(MAINNET_API_URL) {
            (EthChain::Arbitrum, "Arbitrum".to_string())
        } else {
            (EthChain::ArbitrumGoerli, "ArbitrumGoerli".to_string())
        };

        let source = "https://hyperliquid.xyz".to_string();
        let action = serde_json::to_value(Actions::Connect(AgentConnect {
            chain: l1_name,
            agent: Agent {
                source: source.clone(),
                connection_id,
            },
            agent_address: address,
        }))
        .map_err(|e| Error::JsonParse(e.to_string()))?;
        let signature = sign_with_agent(wallet, chain, &source, connection_id)?;
        let timestamp = now_timestamp_ms();
        Ok((key, self.post(action, signature, timestamp).await?))
    }
}
