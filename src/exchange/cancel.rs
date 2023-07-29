use serde::Serialize;

#[derive(Serialize)]
pub struct ClientCancelRequest {
    pub asset: String,
    pub oid: u64,
}

#[derive(Serialize)]
pub(crate) struct CancelRequest {
    pub(crate) asset: u32,
    pub(crate) oid: u64,
}
