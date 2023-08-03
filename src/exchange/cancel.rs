use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClientCancelRequest {
    pub asset: String,
    pub oid: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CancelRequest {
    pub asset: u32,
    pub oid: u64,
}
