use crate::{
    consts::MAINNET_API_URL,
    exchange::{
        actions::{
            AgentConnect, BulkCancel, BulkOrder, UpdateIsolatedMargin, UpdateLeverage, UsdcTransfer,
        },
        cancel::{CancelRequest, CancelRequestCloid},
        ClientCancelRequest, ClientOrderRequest,
    },
    helpers::{generate_random_key, next_nonce, EthChain},
    info::info_client::InfoClient,
    meta::Meta,
    prelude::*,
    req::HttpClient,
    signature::{
        agent::mainnet::Agent, keccak, sign_l1_action, sign_usd_transfer_action, sign_with_agent,
        usdc_transfer::mainnet::UsdTransferSignPayload,
    },
    BaseUrl, BulkCancelCloid, Error, ExchangeResponseStatus,
};
use ethers::{
    abi::AbiEncode,
    signers::{LocalWallet, Signer},
    types::{Signature, H160, H256},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::cancel::ClientCancelRequestCloid;

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
    CancelByCloid(BulkCancelCloid),
    Connect(AgentConnect),
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

impl ExchangeClient {
    pub async fn new(
        client: Option<Client>,
        wallet: LocalWallet,
        base_url: Option<BaseUrl>,
        meta: Option<Meta>,
        vault_address: Option<H160>,
    ) -> Result<ExchangeClient> {
        let client = client.unwrap_or_default();
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

        let timestamp = next_nonce();
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
        let timestamp = next_nonce();

        let mut transformed_orders = Vec::new();

        for order in orders {
            transformed_orders.push(order.convert(&self.coin_to_asset)?);
        }

        let action = Actions::Order(BulkOrder {
            orders: transformed_orders,
            grouping: "na".to_string(),
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;

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
        let is_mainnet = self.http_client.base_url == BaseUrl::Mainnet.get_url();
        let signature = sign_l1_action(wallet, connection_id, is_mainnet)?;

        self.post(action, signature, timestamp).await
    }

    pub async fn cancel_by_cloid(
        &self,
        cancel: ClientCancelRequestCloid,
        wallet: Option<&LocalWallet>,
    ) -> Result<ExchangeResponseStatus> {
        self.bulk_cancel_by_cloid(vec![cancel], wallet).await
    }

    pub async fn bulk_cancel_by_cloid(
        &self,
        cancels: Vec<ClientCancelRequestCloid>,
        wallet: Option<&LocalWallet>,
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
                cloid: cancel.cloid,
            });
        }

        let action = Actions::CancelByCloid(BulkCancelCloid {
            cancels: transformed_cancels,
        });

        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
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

        let timestamp = next_nonce();

        let &asset_index = self.coin_to_asset.get(coin).ok_or(Error::AssetNotFound)?;
        let action = Actions::UpdateLeverage(UpdateLeverage {
            asset: asset_index,
            is_cross,
            leverage,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
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
        let timestamp = next_nonce();

        let &asset_index = self.coin_to_asset.get(coin).ok_or(Error::AssetNotFound)?;
        let action = Actions::UpdateIsolatedMargin(UpdateIsolatedMargin {
            asset: asset_index,
            is_buy: true,
            ntli: amount,
        });
        let connection_id = action.hash(timestamp, self.vault_address)?;
        let action = serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
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
        let timestamp = next_nonce();
        Ok((key, self.post(action, signature, timestamp).await?))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

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
                cloid: Some(cloid.unwrap()),
            }],
            grouping: "na".to_string(),
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
                "e844cafedb695abbc28b3178b136d262327a72bba1012152f3b5b675147e98312d42de83976b05becf768ad882f6f6a1bfa65afadc71f945c2a98473317097ee1b",
                "f360f6173c1d9a8ff2d8677e1fc4cb787122542985129c42e8bce47c5d58f6910ee42b10fd69af0bff0dd484e2cb8d3fa8fecfec13bde5e31f5d3d47d1e5a73f1b"
            ),
            (
                "sl",
                "d10f92a81428c0b57fb619f206bca34ad0cb668be8305306804b27491b4f9c257a87dbd87ad5b6e2bce2ae466b004f7572c5080672ed58cdcb3ffaedcd9de9111c",
                "51b70df3ee8afcdf192390ee79a18b54a8ec92c86653e8ef80b0c90a7cf9850500c6653c4aa2317e7312dfc9b2aeba515d801d7e8af66567539861a6d5eb2d2b1c"
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
