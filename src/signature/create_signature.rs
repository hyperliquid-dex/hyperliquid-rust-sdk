use ethers::{
    abi::AbiEncode,
    core::k256::{
        ecdsa::{signature::hazmat::PrehashSigner, RecoveryId, Signature as RecoverableSignature},
        elliptic_curve::FieldBytes,
        Secp256k1,
    },
    signers::{LocalWallet, Wallet},
    types::{transaction::eip712::Eip712, Signature, H256, U256},
    utils::keccak256,
};

use crate::{
    helpers::EthChain,
    prelude::*,
    signature::{
        agent::{l1, mainnet, testnet},
        usdc_transfer,
    },
    Error,
};

pub(crate) fn keccak(x: impl AbiEncode) -> H256 {
    keccak256(x.encode()).into()
}

pub(crate) fn sign_l1_action(
    wallet: &LocalWallet,
    connection_id: H256,
    is_mainnet: bool,
) -> Result<Signature> {
    sign_with_agent(
        wallet,
        EthChain::Localhost,
        if is_mainnet { "a" } else { "b" },
        connection_id,
    )
}

pub(crate) fn sign_usd_transfer_action(
    wallet: &LocalWallet,
    chain_type: EthChain,
    amount: &str,
    destination: &str,
    timestamp: u64,
) -> Result<Signature> {
    match chain_type {
        EthChain::Localhost => Err(Error::ChainNotAllowed),
        EthChain::Arbitrum => Ok(sign_typed_data(
            &usdc_transfer::mainnet::UsdTransferSignPayload {
                destination: destination.to_string(),
                amount: amount.to_string(),
                time: timestamp,
            },
            wallet,
        )?),
        EthChain::ArbitrumGoerli => Ok(sign_typed_data(
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
    chain_type: EthChain,
    source: &str,
    connection_id: H256,
) -> Result<Signature> {
    match chain_type {
        EthChain::Localhost => sign_typed_data(
            &l1::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
        EthChain::Arbitrum => sign_typed_data(
            &mainnet::Agent {
                source: source.to_string(),
                connection_id,
            },
            wallet,
        ),
        EthChain::ArbitrumGoerli => sign_typed_data(
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
    let signature: Signature = sign_hash(H256::from(encoded), &wallet)?;

    Ok(signature)
}

fn sign_hash(hash: H256, wallet: &LocalWallet) -> Result<Signature> {
    let (recoverable_sig, recovery_id) = wallet
        .signer()
        .sign_prehash(hash.as_ref())
        .map_err(|e| Error::PrehashSigner(e.to_string()))?;

    let v = u8::from(recovery_id) as u64 + 27;

    let r_bytes: FieldBytes<Secp256k1> = recoverable_sig.r().into();
    let s_bytes: FieldBytes<Secp256k1> = recoverable_sig.s().into();
    let r = U256::from_big_endian(r_bytes.as_slice());
    let s = U256::from_big_endian(s_bytes.as_slice());

    Ok(Signature { r, s, v })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn get_wallet() -> Result<LocalWallet> {
        let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
        priv_key
            .parse::<LocalWallet>()
            .map_err(|e| Error::Wallet(e.to_string()))
    }
    #[test]
    fn test_sign_l1_action() -> Result<()> {
        let wallet = get_wallet()?;
        let connection_id =
            H256::from_str("0xde6c4037798a4434ca03cd05f00e3b803126221375cd1e7eaaaf041768be06eb")
                .map_err(|e| Error::GenericParse(e.to_string()))?;

        let expected_mainnet_sig = "fa8a41f6a3fa728206df80801a83bcbfbab08649cd34d9c0bfba7c7b2f99340f53a00226604567b98a1492803190d65a201d6805e5831b7044f17fd530aec7841c";
        assert_eq!(
            sign_l1_action(&wallet, connection_id, true)?.to_string(),
            expected_mainnet_sig
        );
        let expected_testnet_sig = "1713c0fc661b792a50e8ffdd59b637b1ed172d9a3aa4d801d9d88646710fb74b33959f4d075a7ccbec9f2374a6da21ffa4448d58d0413a0d335775f680a881431c";
        assert_eq!(
            sign_l1_action(&wallet, connection_id, false)?.to_string(),
            expected_testnet_sig
        );
        Ok(())
    }

    #[test]
    fn test_sign_usd_transfer_action() -> Result<()> {
        let wallet = get_wallet()?;

        let chain_type = EthChain::ArbitrumGoerli;
        let amount = "1";
        let destination = "0x0D1d9635D0640821d15e323ac8AdADfA9c111414";
        let timestamp = 1690393044548;

        let expected_sig = "78f879e7ae6fbc3184dc304317e602507ac562b49ad9a5db120a41ac181b96ba2e8bd7022526a1827cf4b0ba96384d40ec3a5ed4239499c328081f3d0b394bb61b";
        assert_eq!(
            sign_usd_transfer_action(&wallet, chain_type, amount, destination, timestamp)?
                .to_string(),
            expected_sig
        );
        Ok(())
    }
}
