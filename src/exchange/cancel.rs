use serde::Serialize;

#[derive(Serialize)]
pub struct ClientCancelRequest {
    pub asset: String,
    pub oid: u64,
}

#[derive(Serialize)]
pub struct CancelRequest {
    pub asset: u32,
    pub oid: u64,
}
