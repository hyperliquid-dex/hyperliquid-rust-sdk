use reqwest; 
use crate::{wallet::LocalAccount, meta::Meta, consts, signature, req::post}; 
use serde_json; 
use serde::{Serialize}; 

pub struct Exchange {
    client: reqwest::Client, 
    wallet: LocalAccount, 
    base_url: String, 
    meta: Meta, 
    vault_address: Option<String>, 
}

#[derive(Serialize)]
pub struct ExchangePayload {
    action: serde_json::Value, 
    signature: signature::Signature, 
    nonce: u64, 
    vault_address: Option<String>,
}

impl Exchange {
    pub fn new (optional_client: Option<reqwest::Client>, wallet: LocalAccount, base_url: Option<String>, meta: Option<Meta>, vault_address: Option<String>) -> Self {
        let client = optional_client.unwrap_or_else(|| reqwest::Client::new()); 

        let unwrapped_base_url = base_url.unwrap_or_else(|| consts::MAINNET_API_URL.to_owned()); 

        Exchange {
            client: client, 
            wallet: wallet, 
            base_url: unwrapped_base_url, 
            meta: meta.unwrap(), 
            vault_address: vault_address, 
        }
    }

    async fn post_action (&self, action: serde_json::Value, signature: signature::Signature, nonce: u64) -> String {
        let exchange_payload = ExchangePayload{
            action: action, 
            signature: signature, 
            nonce: nonce, 
            vault_address: self.vault_address.clone(), 
        }; 
        let res = serde_json::to_string(&exchange_payload).unwrap();
        let url = self.base_url.clone() + "/exchange"; 
        return post(&self.client, &url, res).await; 
    }

}