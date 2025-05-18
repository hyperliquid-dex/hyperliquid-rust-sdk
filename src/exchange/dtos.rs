use ethers::types::H256;
use serde::{Deserialize, Serialize};

use super::Actions;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageResponse {
    pub action: Actions,
    pub message: H256,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
pub struct SpotTransferRequest {
    pub amount: String,
    pub destination: String,
    pub token: String,
    pub signature_chain_id: i64,
}
