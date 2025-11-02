use crate::WsManager;
use crate::{
    exchange::{order::OrderRequest, BuilderInfo},
    helpers::next_nonce,
    prelude::*,
    signature::{sign_l1_action,sign_typed_data},
    BulkOrder,SpotSend, Error,
};
use alloy::primitives::{keccak256, Address, Signature, B256, U256};
use alloy::signers::local::PrivateKeySigner;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum OrderStatus {
    Filled { filled: OrderFillDetails },
    Resting { resting: OrderRestingDetails },
    Error { error: String },
}

#[derive(Debug, Deserialize)]
pub(crate) struct OrderFillDetails {
    pub oid: u64,
    pub total_sz: Option<String>,
    pub avg_px: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OrderRestingDetails {
    pub oid: u64,
}

// Use #[serde(untagged)] to remove the enum wrapper
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub(crate) enum Actions {
    Order(BulkOrder),
    SpotSend(SpotSend),
}

#[derive(Debug, Clone, Deserialize)]
struct SignatureData {
    r: U256,
    s: U256,
    v: u8,
}

impl Serialize for SignatureData {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SignatureData", 3)?;
        state.serialize_field("r", &self.r)?;
        state.serialize_field("s", &self.s)?;
        state.serialize_field("v", &self.v)?;
        state.end()
    }
}

impl From<Signature> for SignatureData {
    fn from(sig: Signature) -> Self {
        SignatureData {
            r: sig.r().into(),
            s: sig.s().into(),
            v: if sig.v() { 28 } else { 27 } as u8,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExchangePayload {
    action: serde_json::Value,
    signature: SignatureData,
    nonce: u64,
    vault_address: Option<Address>,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SignedAction {
    action: Actions,
    nonce: u64,
    signature: SignatureData,
    #[serde(skip_serializing_if = "Option::is_none")]
    vault_address: Option<String>,
}

pub async fn bulk_order_with_builder(
    orders: Vec<OrderRequest>,
    wallet: Option<&PrivateKeySigner>,
    mut builder: Option<BuilderInfo>,
    vault_address: Option<Address>,
    nonce: u64,
) -> Result<serde_json::Value> {
    let wallet = wallet
        .as_ref()
        .ok_or(Error::JsonParse("Wallet not provided".to_string()))?;

    if let Some(builder) = &mut builder {
        builder.builder = builder.builder.to_lowercase();
    } else {
        builder = None;
    }

    let mut transformed_orders = Vec::new();

    for order in orders {
        transformed_orders.push(order);
    }

    // Create the action with proper type field
    let action = Actions::Order(BulkOrder {
        orders: transformed_orders,
        grouping: "na".to_string(),
        builder: builder,
    });
    let action_value =
        serde_json::to_value(&action).map_err(|e| Error::JsonParse(e.to_string()))?;
    println!("Action: {:#?}", action_value);
    // Hash the Actions (this serializes to MessagePack)
    let connection_id = action.hash(nonce, vault_address)?;
    println!("Connection ID: {:#?}", connection_id);

    let signature = sign_l1_action(wallet, connection_id, true).unwrap();
    let exchange_payload = ExchangePayload {
        action: action_value,
        signature: signature.into(),
        nonce: nonce,
        vault_address: vault_address,
    };

    let payload =
        serde_json::to_value(&exchange_payload).map_err(|e| Error::JsonParse(e.to_string()))?;
    return Ok(payload);
}

 pub async fn spot_transfer(
        amount: &str,
        destination: &str,
        token: &str,
        wallet: PrivateKeySigner,
        nonce: u64,
    ) -> Result<serde_json::Value> {

        let spot_send = SpotSend {
            signature_chain_id: 421614,
            hyperliquid_chain: "Mainnet".to_string(),
            destination: destination.to_string(),
            amount: amount.to_string(),
            time: nonce,
            token: token.to_string(),
        };
        let signature = sign_typed_data(&spot_send, &wallet)?;
        let action = serde_json::to_value(Actions::SpotSend(spot_send))
            .map_err(|e| Error::JsonParse(e.to_string()))?;
        
        let exchange_payload = ExchangePayload {
            action: action,
            signature: signature.into(),
            nonce: nonce,
            vault_address: None,
        };
         let payload = serde_json::to_value(&exchange_payload).map_err(|e| Error::JsonParse(e.to_string()))?;
            return Ok(payload);
    }



#[cfg(test)]
mod tests {

    use super::*;
    use crate::helpers::next_nonce;
    use crate::{exchange::order::Limit, Order};
    use alloy::signers::local::PrivateKeySigner;
    use std::{str::FromStr, time::Duration};

    #[tokio::test]
    async fn test_send_order() {
        let nonce = next_nonce();
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();

        let ws_url = "wss://api.hyperliquid.xyz/ws";

        let private_key = "";
        let wallet = PrivateKeySigner::from_str(private_key).expect("Invalid private key");

        println!("Creating WsManager...");
        let mut ws_manager = WsManager::new(ws_url.to_string(), true)
            .await
            .expect("Failed to create WsManager");

        println!("Waiting for WebSocket connection to stabilize...");
        tokio::time::sleep(Duration::from_secs(2)).await;

        let order = OrderRequest {
            asset: 10151,
            is_buy: false,
            limit_px: "3900".to_string(),
            sz: "0.004".to_string(),
            reduce_only: false,
            order_type: Order::Limit(Limit {
                tif: "Gtc".to_string(),
            }),
            cloid: None,
        };

        let builder = None;

        println!("Sending order...");
        let payload = bulk_order_with_builder(vec![order], Some(&wallet), builder, None, nonce)
            .await
            .unwrap();

        let result = ws_manager.post(payload, nonce).await;
        match result {
            Ok(response) => {
                println!(
                    "Full Response: {}",
                    serde_json::to_string_pretty(&response).unwrap()
                );
            }
            Err(e) => {
                eprintln!("Error sending order: {:?}", e);
            }
        }
    }

     async fn test_spot_transfer() {
        let nonce = next_nonce();
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .try_init();

        let ws_url = "wss://api.hyperliquid.xyz/ws";

        let private_key = "";
        let wallet = PrivateKeySigner::from_str(private_key).expect("Invalid private key");

        println!("Creating WsManager...");
        let mut ws_manager = WsManager::new(ws_url.to_string(), true)
            .await
            .expect("Failed to create WsManager");

        println!("Waiting for WebSocket connection to stabilize...");
        tokio::time::sleep(Duration::from_secs(2)).await;

        let amount = "0.004215951";
        let destination = "0x20000000000000000000000000000000000000dd";
        let token = "UETH:0xe1edd30daaf5caac3fe63569e24748da";

        println!("Sending spot transfer...");
        let payload = spot_transfer(amount, destination, token, wallet, nonce)
            .await
            .unwrap();

        let result = ws_manager.post(payload, nonce).await;
        println!("Result: {:#?}", result);
        match result {
            Ok(response) => {
                println!(
                    "Full Response: {}",
                    serde_json::to_string_pretty(&response).unwrap()
                );
            }
            Err(e) => {
                eprintln!("Error sending order: {:?}", e);
            }
        }
    }

}
