use ethers::{
    abi::AbiEncode,
    core::k256::{
        ecdsa::{recoverable, signature::DigestSigner},
        elliptic_curve::FieldBytes,
        Secp256k1,
    },
    signers::LocalWallet,
    types::{transaction::eip712::Eip712, Signature, H256, U256},
    utils::keccak256,
};

use crate::{
    helpers::ChainType,
    prelude::*,
    proxy_digest::Sha256Proxy,
    signature::{
        agent::{l1, mainnet, testnet},
        usdc_transfer,
    },
    Error,
};

pub(crate) fn keccak(x: impl AbiEncode) -> H256 {
    keccak256(x.encode()).into()
}

pub(crate) fn sign_l1_action(wallet: &LocalWallet, connection_id: H256) -> Result<Signature> {
    sign_with_agent(wallet, ChainType::Arbitrum, "a", connection_id)
}

pub(crate) fn sign_usd_transfer_action(
    wallet: &LocalWallet,
    chain_type: ChainType,
    amount: &str,
    destination: &str,
    timestamp: u64,
) -> Result<Signature> {
    match chain_type {
        ChainType::Arbitrum => Err(Error::ChainNotAllowed),
        ChainType::HyperliquidMainnet => Ok(sign_typed_data(
            &usdc_transfer::mainnet::UsdTransferSignPayload {
                destination: destination.to_string(),
                amount: amount.to_string(),
                time: timestamp,
            },
            wallet,
        )?),
        ChainType::HyperliquidTestnet => Ok(sign_typed_data(
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
) -> Result<Signature> {
    match chain_type {
        ChainType::Arbitrum => sign_typed_data(
            &l1::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
        ChainType::HyperliquidMainnet => sign_typed_data(
            &mainnet::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
        ChainType::HyperliquidTestnet => sign_typed_data(
            &testnet::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
    }
}

fn sign_typed_data<T: Eip712>(payload: &T, wallet: &LocalWallet) -> Result<Signature> {
    let encoded = payload
        .encode_eip712()
        .map_err(|e| Error::Eip712(e.to_string()))?;

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
