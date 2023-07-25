use crate::{
    consts::{MAINNET_API_URL, ZERO_ADDRESS},
    errors::AssetNotFoundError,
    exchange::actions::{UpdateLeverage, UsdcTransfer},
    helpers::{get_timestamp_ms, ChainType},
    info::info_client::InfoClient,
    meta::Meta,
    req::HttpClient,
    signature::{
        keccak, sign_l1_action, sign_usd_transfer_action,
        usdc_transfer::mainnet::UsdTransferSignPayload,
    },
};
use ethers::{
    signers::LocalWallet,
    types::{Signature, H160},
};
use reqwest::Client;
use serde::Serialize;
use std::{collections::HashMap, error::Error, str::FromStr};

pub struct ExchangeClient<'a> {
    pub http_client: HttpClient<'a>,
    pub wallet: LocalWallet,
    pub meta: Meta,
    pub vault_address: Option<&'a str>,
    pub coin_to_asset: HashMap<String, u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExchangePayload<'a> {
    action: serde_json::Value,
    signature: Signature,
    nonce: u64,
    vault_address: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum Actions {
    UsdTransfer(UsdcTransfer),
    UpdateLeverage(UpdateLeverage),
}

impl<'a> ExchangeClient<'a> {
    pub async fn new(
        client: Option<Client>,
        wallet: LocalWallet,
        base_url: Option<&'a str>,
        meta: Option<Meta>,
        vault_address: Option<&'a str>,
    ) -> Result<ExchangeClient<'a>, Box<dyn Error>> {
        let client = client.unwrap_or_else(Client::new);
        let base_url = base_url.unwrap_or(MAINNET_API_URL);

        let retrieved_meta;
        if let Some(unwrapped_meta) = meta {
            retrieved_meta = unwrapped_meta;
        } else {
            let info = InfoClient::new(None, Some(base_url));
            retrieved_meta = info.meta().await?;
        }

        let mut coin_to_asset = HashMap::new();
        for (asset_ind, asset) in retrieved_meta.universe.iter().enumerate() {
            coin_to_asset.insert(asset.name.clone(), asset_ind as u32);
        }

        Ok(ExchangeClient {
            wallet,
            meta: retrieved_meta,
            vault_address,
            http_client: HttpClient { client, base_url },
            coin_to_asset,
        })
    }

    async fn post(
        &self,
        action: serde_json::Value,
        signature: Signature,
        nonce: u64,
    ) -> Result<String, Box<dyn Error>> {
        let exchange_payload = ExchangePayload {
            action,
            signature,
            nonce,
            vault_address: self.vault_address,
        };
        let res = serde_json::to_string(&exchange_payload).unwrap();
        self.http_client.post("/exchange", res).await
    }

    pub async fn usdc_transfer(
        &self,
        amount: &str,
        destination: &str,
    ) -> Result<String, Box<dyn Error>> {
        let chain_type;
        let l1_name;
        if self.http_client.base_url.eq(MAINNET_API_URL) {
            chain_type = ChainType::Mainnet;
            l1_name = "Arbitrum".to_string();
        } else {
            chain_type = ChainType::Testnet;
            l1_name = "ArbitrumGoerli".to_string();
        }

        let timestamp = get_timestamp_ms();
        let payload = serde_json::to_value(UsdTransferSignPayload {
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: timestamp,
        })?;
        let action = serde_json::to_value(Actions::UsdTransfer(UsdcTransfer {
            chain: l1_name,
            payload,
        }))?;

        let signature =
            sign_usd_transfer_action(&self.wallet, chain_type, amount, destination, timestamp)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn update_leverage(
        &self,
        leverage: u32,
        coin: &str,
        is_cross: bool,
    ) -> Result<String, Box<dyn Error>> {
        let timestamp = get_timestamp_ms();
        let vault_address = H160::from_str(self.vault_address.unwrap_or(ZERO_ADDRESS))?;

        if let Some(&asset_index) = self.coin_to_asset.get(coin) {
            let connection_id = keccak((asset_index, is_cross, leverage, vault_address, timestamp));

            let action = serde_json::to_value(Actions::UpdateLeverage(UpdateLeverage {
                asset: asset_index,
                is_cross,
                leverage,
            }))?;

            let signature = sign_l1_action(&self.wallet, connection_id)?;
            self.post(action, signature, timestamp).await
        } else {
            Err(Box::new(AssetNotFoundError))
        }
    }
}
