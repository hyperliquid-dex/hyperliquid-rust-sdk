use crate::{prelude::*, signature::agent::l1, Error};
use ethers::types::transaction::eip712::Eip712;
use ethers::types::H256;

#[cfg(not(feature = "testnet"))]
const SOURCE: &str = "a";

#[cfg(feature = "testnet")]
const SOURCE: &str = "b";

pub fn encode_l1_action(connection_id: H256) -> Result<H256> {
    let payload = &l1::Agent {
        source: SOURCE.to_string(),
        connection_id,
    };
    let encoded = payload
        .encode_eip712()
        .map_err(|e| Error::Eip712(e.to_string()))?;

    let action = H256::from(encoded);
    Ok(action)
}
