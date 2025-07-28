use serde::{Deserialize, Serialize};

use super::{order::OrderRequest, ClientOrderRequest};

#[derive(Debug)]
pub struct ClientModifyRequest {
    pub oid: u64,
    pub order: ClientOrderRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModifyRequest {
    pub oid: u64,
    pub order: OrderRequest,
}
