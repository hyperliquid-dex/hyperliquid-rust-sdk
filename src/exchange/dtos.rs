use ethers::types::H256;
use serde::{Deserialize, Serialize};

use super::Actions;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageResponse {
    pub action: Actions,
    pub message: H256,
    pub nonce: u64,
}


