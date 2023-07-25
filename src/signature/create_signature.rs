use ethers::{
    abi::AbiEncode,
    core::k256::{
        ecdsa::{recoverable, signature::DigestSigner},
        elliptic_curve::FieldBytes,
        Secp256k1,
    },
    signers::LocalWallet,
    types::{
        transaction::eip712::{Eip712, Eip712Error},
        Signature, H256, U256,
    },
    utils::keccak256,
};

use crate::helpers::ChainType;
use crate::proxy_digest::Sha256Proxy;
use crate::signature::{
    agent::{l1, mainnet, testnet},
    usdc_transfer,
};

use crate::errors::ChainError;

use std::error::Error;

pub(crate) fn keccak(x: impl AbiEncode) -> H256 {
    keccak256(x.encode()).into()
}

pub(crate) fn sign_l1_action(
    wallet: &LocalWallet,
    connection_id: H256,
) -> Result<Signature, Box<dyn Error>> {
    Ok(sign_with_agent(wallet, ChainType::L1, "a", connection_id)?)
}

pub(crate) fn sign_usd_transfer_action(
    wallet: &LocalWallet,
    chain_type: ChainType,
    amount: &str,
    destination: &str,
    timestamp: u64,
) -> Result<Signature, Box<dyn Error>> {
    match chain_type {
        ChainType::L1 => Err(Box::new(ChainError)),
        ChainType::Mainnet => Ok(sign_typed_data(
            &usdc_transfer::mainnet::UsdTransferSignPayload {
                destination: destination.to_string(),
                amount: amount.to_string(),
                time: timestamp,
            },
            wallet,
        )?),
        ChainType::Testnet => Ok(sign_typed_data(
            &usdc_transfer::testnet::UsdTransferSignPayload {
                destination: destination.to_string(),
                amount: amount.to_string(),
                time: timestamp,
            },
            wallet,
        )?),
    }
}

pub(crate) fn sign_with_agent(
    wallet: &LocalWallet,
    chain_type: ChainType,
    source: &str,
    connection_id: H256,
) -> Result<Signature, Eip712Error> {
    match chain_type {
        ChainType::L1 => sign_typed_data(
            &l1::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
        ChainType::Mainnet => sign_typed_data(
            &mainnet::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
        ChainType::Testnet => sign_typed_data(
            &testnet::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
    }
}

fn sign_typed_data<T: Eip712>(payload: &T, wallet: &LocalWallet) -> Result<Signature, Eip712Error> {
    let encoded = payload
        .encode_eip712()
        .map_err(|e| Eip712Error::Message(e.to_string()))?;

    Ok(sign_hash(H256::from(encoded), wallet))
}

fn sign_hash(hash: H256, wallet: &LocalWallet) -> Signature {
    let recoverable_sig: recoverable::Signature =
        wallet.signer().sign_digest(Sha256Proxy::from(hash));

    let v = u8::from(recoverable_sig.recovery_id()) as u64 + 27;

    let r_bytes: FieldBytes<Secp256k1> = recoverable_sig.r().into();
    let s_bytes: FieldBytes<Secp256k1> = recoverable_sig.s().into();
    let r = U256::from_big_endian(r_bytes.as_slice());
    let s = U256::from_big_endian(s_bytes.as_slice());

    Signature { r, s, v }
}
