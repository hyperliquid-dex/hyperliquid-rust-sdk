use crate::{consts::MAINNET_API_URL, meta::Meta, req::ClientAndBaseUrl, signature::Signature, wallet::LocalAccount};
use reqwest::Client;
use serde::Serialize;
use serde_json;
use std::error::Error;

pub struct Exchange {
    pub client_and_base_url: ClientAndBaseUrl,
    pub wallet: LocalAccount,
    pub meta: Option<Meta>,
    pub vault_address: Option<String>,
}

#[derive(Serialize)]
pub struct ExchangePayload {
    action: serde_json::Value,
    signature: Signature,
    nonce: u64,
    vault_address: Option<String>,
}

impl Exchange {
    pub fn new(
        client: Option<Client>,
        wallet: LocalAccount,
        base_url: Option<String>,
        meta: Option<Meta>,
        vault_address: Option<String>,
    ) -> Self {
        let client = client.unwrap_or_else(Client::new);
        let base_url = base_url.unwrap_or_else(|| MAINNET_API_URL.to_owned());

        Exchange {
            wallet,
            meta,
            vault_address,
            client_and_base_url: ClientAndBaseUrl { client, base_url },
        }
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
            vault_address: self.vault_address.clone(),
        };
        let res = serde_json::to_string(&exchange_payload).unwrap();
        self.client_and_base_url
            .post("/exchange".to_string(), res)
            .await
    }
}
