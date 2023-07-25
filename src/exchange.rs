use crate::{consts::MAINNET_API_URL, meta::Meta, req::HttpClient, signature::Signature};
use ethers::signers::LocalWallet;
use reqwest::Client;
use serde::Serialize;
use std::error::Error;

pub struct ExchangeClient<'a> {
    pub http_client: HttpClient<'a>,
    pub wallet: LocalWallet,
    pub meta: Option<Meta>,
    pub vault_address: Option<&'a str>,
}

#[derive(Serialize)]
struct ExchangePayload<'a> {
    action: serde_json::Value,
    signature: Signature,
    nonce: u64,
    vault_address: Option<&'a str>,
}

impl<'a> ExchangeClient<'a> {
    pub fn new(
        client: Option<Client>,
        wallet: LocalWallet,
        base_url: Option<&'a str>,
        meta: Option<Meta>,
        vault_address: Option<&'a str>,
    ) -> Self {
        let client = client.unwrap_or_else(Client::new);
        let base_url = base_url.unwrap_or(MAINNET_API_URL);

        ExchangeClient {
            wallet,
            meta,
            vault_address,
            http_client: HttpClient { client, base_url },
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
            vault_address: self.vault_address,
        };
        let res = serde_json::to_string(&exchange_payload).unwrap();
        self.http_client.post("/exchange", res).await
    }
}
