use super::{order::OrderRequest, ClientOrderRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ClientModifyRequest {
    pub oid: Oid,
    pub order: ClientOrderRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModifyRequest {
    pub oid: Oid,
    pub order: OrderRequest,
}

// Oid can be provided as either a number or string per API docs
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Oid {
    Num(u64),
    Str(String),
}

impl From<u64> for Oid {
    fn from(value: u64) -> Self {
        Oid::Num(value)
    }
}

impl From<String> for Oid {
    fn from(value: String) -> Self {
        Oid::Str(value)
    }
}
