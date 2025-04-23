use crate::proxy_digest::Sha256Proxy;
use crate::{prelude::*, signature::agent::l1, Error};
use ethers::{
    core::k256::{elliptic_curve::FieldBytes, Secp256k1},
    signers::LocalWallet,
    types::{transaction::eip712::Eip712, Signature, H256, U256},
};

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

pub fn sign_hash(hash: H256) -> Result<Signature> {
    let wallet = get_wallet()?;
    let (sig, rec_id) = wallet
        .signer()
        .sign_digest_recoverable(Sha256Proxy::from(hash))
        .map_err(|e| Error::SignatureFailure(e.to_string()))?;

    let v = u8::from(rec_id) as u64 + 27;

    let r_bytes: FieldBytes<Secp256k1> = sig.r().into();
    let s_bytes: FieldBytes<Secp256k1> = sig.s().into();
    let r = U256::from_big_endian(r_bytes.as_slice());
    let s = U256::from_big_endian(s_bytes.as_slice());

    Ok(Signature { r, s, v })
}

fn get_wallet() -> Result<LocalWallet> {
    //let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
    let priv_key = "844027a8d05f609402abbc96f6f9cd6ea2b8550e7650274191ad3d8f106fda2c";
    priv_key
        .parse::<LocalWallet>()
        .map_err(|e| Error::Wallet(e.to_string()))
}
