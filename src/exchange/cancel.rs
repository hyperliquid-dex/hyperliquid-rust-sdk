use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ClientCancelRequest {
    pub asset: u32,
    pub oid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelRequest {
    #[serde(rename = "a", alias = "asset")]
    pub asset: u32,
    #[serde(rename = "o", alias = "oid")]
    pub oid: u64,
}

#[derive(Debug)]
pub struct ClientCancelRequestCloid {
    pub asset: u32,
    pub cloid: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelRequestCloid {
    pub asset: u32,
    pub cloid: String,
}
