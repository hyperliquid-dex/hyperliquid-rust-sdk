use super::{order::OrderRequest, ClientOrderRequest};
use serde::{Deserialize, Serialize};

pub struct ClientModifyRequest {
    pub oid: u64,
    pub order: ClientOrderRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModifyRequest {
    pub oid: u64,
    pub order: OrderRequest,
}
