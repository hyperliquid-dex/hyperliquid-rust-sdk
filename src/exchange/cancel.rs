use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientCancelRequest {
    pub asset: String,
    pub oid: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelRequest {
    pub asset: u32,
    pub oid: u64,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct ClientCancelRequestCloid {
    pub asset: String,
    pub cloid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancelRequestCloid {
    pub asset: u32,
    pub cloid: String,
}