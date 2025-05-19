use crate::helpers::uuid_to_hex_string;

use super::{order::OrderRequest, ClientOrderRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ClientModifyId {
    Oid(u64),
    Cloid(Uuid),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SerializedClientId {
    U64(u64),
    Str(String),
}

impl From<u64> for ClientModifyId {
    fn from(oid: u64) -> Self {
        ClientModifyId::Oid(oid)
    }
}

impl From<Uuid> for ClientModifyId {
    fn from(cloid: Uuid) -> Self {
        ClientModifyId::Cloid(cloid)
    }
}

impl From<ClientModifyId> for SerializedClientId {
    fn from(client_id: ClientModifyId) -> Self {
        match client_id {
            ClientModifyId::Oid(oid) => SerializedClientId::U64(oid),
            ClientModifyId::Cloid(cloid) => SerializedClientId::Str(uuid_to_hex_string(cloid)),
        }
    }
}

#[derive(Debug)]
pub struct ClientModifyRequest {
    pub oid: ClientModifyId,
    pub order: ClientOrderRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModifyRequest {
    pub oid: SerializedClientId,
    pub order: OrderRequest,
}
