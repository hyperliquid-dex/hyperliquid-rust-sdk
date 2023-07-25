use serde::Serialize;

use ethers::{abi::AbiEncode, types::H256, utils::keccak256};

#[derive(Serialize)]
pub(crate) struct Signature {
    pub(crate) r: String,
    pub(crate) s: String,
    pub(crate) v: String,
}

fn keccak(x: impl AbiEncode) -> H256 {
    keccak256(x.encode()).into()
}
