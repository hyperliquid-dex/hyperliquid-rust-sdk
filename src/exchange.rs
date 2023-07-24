use crate::{consts, meta::Meta, req::post, signature, wallet::LocalAccount};
use reqwest;
use serde::Serialize;
use serde_json;

pub struct Exchange {
    pub client: reqwest::Client,
    pub wallet: LocalAccount,
    pub base_url: String,
    pub meta: Meta,
    pub vault_address: Option<String>,
}

#[derive(Serialize)]
pub struct ExchangePayload {
    action: serde_json::Value,
    signature: signature::Signature,
    nonce: u64,
    vault_address: Option<String>,
}

impl Exchange {
    pub fn new(
        optional_client: Option<reqwest::Client>,
        wallet: LocalAccount,
        base_url: Option<String>,
        meta: Option<Meta>,
        vault_address: Option<String>,
    ) -> Self {
        let client = optional_client.unwrap_or_else(reqwest::Client::new);

        let unwrapped_base_url = base_url.unwrap_or_else(|| consts::MAINNET_API_URL.to_owned());

        Exchange {
            client,
            wallet,
            base_url: unwrapped_base_url,
            meta: meta.unwrap(),
            vault_address,
        }
    }

    async fn post_action(
        &self,
        action: serde_json::Value,
        signature: signature::Signature,
        nonce: u64,
    ) -> String {
        let exchange_payload = ExchangePayload {
            action,
            signature,
            nonce,
            vault_address: self.vault_address.clone(),
        };
        let res = serde_json::to_string(&exchange_payload).unwrap();
        let url = self.base_url.clone() + "/exchange";
        post(&self.client, &url, res).await
    }
}
