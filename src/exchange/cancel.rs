use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct ClientCancelRequest {
    pub asset: String,
    pub oid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelRequest {
    #[serde(rename = "a", alias = "asset")]
    pub asset: u32,
    #[serde(rename = "o", alias = "oid")]
    pub oid: u64,
}

pub struct ClientCancelRequestCloid {
    pub asset: String,
    pub cloid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelRequestCloid {
    pub asset: u32,
    pub cloid: String,
}
