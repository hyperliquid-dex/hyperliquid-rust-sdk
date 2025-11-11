use crate::{eip712::Eip712, prelude::*, signature::agent::l1, Error};
use alloy::{
    primitives::B256,
    signers::{local::PrivateKeySigner, Signature, SignerSync},
};

pub(crate) fn sign_l1_action(
    wallet: &PrivateKeySigner,
    connection_id: B256,
    is_mainnet: bool,
) -> Result<Signature> {
    let source = if is_mainnet { "a" } else { "b" }.to_string();
    let payload = l1::Agent {
        source,
        connectionId: connection_id,
    };
    sign_typed_data(&payload, wallet)
}

pub(crate) fn sign_typed_data<T: Eip712>(
    payload: &T,
    wallet: &PrivateKeySigner,
) -> Result<Signature> {
    wallet
        .sign_hash_sync(&payload.eip712_signing_hash())
        .map_err(|e| Error::SignatureFailure(e.to_string()))
}

/// Sign an L1 action with multiple wallets for multi-sig
pub(crate) fn sign_l1_action_multi_sig(
    wallets: &[PrivateKeySigner],
    connection_id: B256,
    is_mainnet: bool,
) -> Result<Vec<Signature>> {
    let mut signatures = Vec::with_capacity(wallets.len());
    for wallet in wallets {
        signatures.push(sign_l1_action(wallet, connection_id, is_mainnet)?);
    }
    Ok(signatures)
}

/// Sign typed data with multiple wallets for multi-sig
pub(crate) fn sign_typed_data_multi_sig<T: Eip712>(
    payload: &T,
    wallets: &[PrivateKeySigner],
) -> Result<Vec<Signature>> {
    let mut signatures = Vec::with_capacity(wallets.len());
    for wallet in wallets {
        signatures.push(sign_typed_data(payload, wallet)?);
    }
    Ok(signatures)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sign_multi_sig_l1_action_payload(
    wallets: &[PrivateKeySigner],
    action: &serde_json::Value,
    multi_sig_user: alloy::primitives::Address,
    outer_signer: alloy::primitives::Address,
    vault_address: Option<alloy::primitives::Address>,
    nonce: u64,
    expires_after: Option<u64>,
    is_mainnet: bool,
) -> Result<Vec<Signature>> {
    let multi_sig_user_str = format!("{:?}", multi_sig_user).to_lowercase();
    let outer_signer_str = format!("{:?}", outer_signer).to_lowercase();

    let envelope = serde_json::json!([multi_sig_user_str, outer_signer_str, action]);

    let mut bytes =
        rmp_serde::to_vec_named(&envelope).map_err(|e| Error::RmpParse(e.to_string()))?;

    bytes.extend(nonce.to_be_bytes());

    if let Some(vault_address) = vault_address {
        bytes.push(1);
        bytes.extend(vault_address.as_slice());
    } else {
        bytes.push(0);
    }

    if let Some(expires_after) = expires_after {
        bytes.push(0);
        bytes.extend(expires_after.to_be_bytes());
    }

    let connection_id = alloy::primitives::keccak256(bytes);

    sign_l1_action_multi_sig(wallets, connection_id, is_mainnet)
}

/// Sign a multi-sig action with the outer signer's wallet
/// This is called after the inner signatures have been collected
/// The function:
/// 1. Removes the "type" field from the multi_sig_action
/// 2. Computes the action hash using msgpack + nonce + vault_address + expires_after
/// 3. Creates and signs the MultiSigEnvelope
pub(crate) fn sign_multi_sig_action(
    wallet: &PrivateKeySigner,
    multi_sig_action: &serde_json::Value,
    vault_address: Option<alloy::primitives::Address>,
    nonce: u64,
    expires_after: Option<u64>,
    is_mainnet: bool,
) -> Result<Signature> {
    use crate::exchange::actions::MultiSigEnvelope;
    use alloy::primitives::keccak256;

    // Remove the "type" field before hashing (as per Python SDK)
    let mut action_without_type = multi_sig_action.clone();
    if let Some(obj) = action_without_type.as_object_mut() {
        obj.remove("type");
    }

    let mut bytes = rmp_serde::to_vec_named(&action_without_type)
        .map_err(|e| Error::RmpParse(e.to_string()))?;
    println!("{}", action_without_type);

    bytes.extend(nonce.to_be_bytes());

    if let Some(vault_address) = vault_address {
        bytes.push(1);
        bytes.extend(vault_address.as_slice());
    } else {
        bytes.push(0);
    }

    if let Some(expires_after) = expires_after {
        bytes.push(0);
        bytes.extend(expires_after.to_be_bytes());
    }

    let multi_sig_action_hash = keccak256(bytes);

    // Create the envelope to sign
    let hyperliquid_chain = if is_mainnet {
        "Mainnet".to_string()
    } else {
        "Testnet".to_string()
    };

    let envelope = MultiSigEnvelope {
        signature_chain_id: 421614, // Always use this chain ID for multi-sig
        hyperliquid_chain,
        multi_sig_action_hash,
        nonce,
    };

    sign_typed_data(&envelope, wallet)
}

/// Sign a multi-sig user-signed action payload with a single wallet
/// This is used to collect individual signatures from multi-sig participants
///
/// # Arguments
/// * `wallet` - The wallet of the multi-sig participant
/// * `action` - The SendAsset or other user-signed action to sign
///
/// # Returns
/// A single signature that can be collected and combined with others
pub fn sign_multi_sig_user_signed_action_single<T: Eip712>(
    wallet: &PrivateKeySigner,
    action: &T,
) -> Result<Signature> {
    sign_typed_data(action, wallet)
}

/// Sign a multi-sig L1 action payload with a single wallet
/// This is used to collect individual signatures from multi-sig participants for L1 actions
///
/// # Arguments
/// * `wallet` - The wallet of the multi-sig participant
/// * `action` - The action to sign (e.g., order, cancel, etc.)
/// * `multi_sig_user` - The address of the multi-sig user
/// * `outer_signer` - The address of the wallet that will submit the transaction
/// * `vault_address` - Optional vault address
/// * `nonce` - The nonce for this action
/// * `expires_after` - Optional expiration timestamp
/// * `is_mainnet` - Whether this is for mainnet or testnet
///
/// # Returns
/// A single signature that can be collected and combined with others
#[allow(clippy::too_many_arguments)]
pub fn sign_multi_sig_l1_action_single(
    wallet: &PrivateKeySigner,
    action: &serde_json::Value,
    multi_sig_user: alloy::primitives::Address,
    outer_signer: alloy::primitives::Address,
    vault_address: Option<alloy::primitives::Address>,
    nonce: u64,
    expires_after: Option<u64>,
    is_mainnet: bool,
) -> Result<Signature> {
    let multi_sig_user_str = format!("{:?}", multi_sig_user).to_lowercase();
    let outer_signer_str = format!("{:?}", outer_signer).to_lowercase();

    let envelope = serde_json::json!([multi_sig_user_str, outer_signer_str, action]);

    let mut bytes =
        rmp_serde::to_vec_named(&envelope).map_err(|e| Error::RmpParse(e.to_string()))?;

    bytes.extend(nonce.to_be_bytes());

    if let Some(vault_address) = vault_address {
        bytes.push(1);
        bytes.extend(vault_address.as_slice());
    } else {
        bytes.push(0);
    }

    if let Some(expires_after) = expires_after {
        bytes.push(0);
        bytes.extend(expires_after.to_be_bytes());
    }

    let connection_id = alloy::primitives::keccak256(bytes);

    sign_l1_action(wallet, connection_id, is_mainnet)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{UsdSend, Withdraw3};

    fn get_wallet() -> Result<PrivateKeySigner> {
        let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
        priv_key
            .parse::<PrivateKeySigner>()
            .map_err(|e| Error::Wallet(e.to_string()))
    }

    #[test]
    fn test_sign_l1_action() -> Result<()> {
        let wallet = get_wallet()?;
        let connection_id =
            B256::from_str("0xde6c4037798a4434ca03cd05f00e3b803126221375cd1e7eaaaf041768be06eb")
                .map_err(|e| Error::GenericParse(e.to_string()))?;

        let expected_mainnet_sig = "0xfa8a41f6a3fa728206df80801a83bcbfbab08649cd34d9c0bfba7c7b2f99340f53a00226604567b98a1492803190d65a201d6805e5831b7044f17fd530aec7841c";
        assert_eq!(
            sign_l1_action(&wallet, connection_id, true)?.to_string(),
            expected_mainnet_sig
        );
        let expected_testnet_sig = "0x1713c0fc661b792a50e8ffdd59b637b1ed172d9a3aa4d801d9d88646710fb74b33959f4d075a7ccbec9f2374a6da21ffa4448d58d0413a0d335775f680a881431c";
        assert_eq!(
            sign_l1_action(&wallet, connection_id, false)?.to_string(),
            expected_testnet_sig
        );
        Ok(())
    }

    #[test]
    fn test_sign_usd_transfer_action() -> Result<()> {
        let wallet = get_wallet()?;

        let usd_send = UsdSend {
            signature_chain_id: 421614,
            hyperliquid_chain: "Testnet".to_string(),
            destination: "0x0D1d9635D0640821d15e323ac8AdADfA9c111414".to_string(),
            amount: "1".to_string(),
            time: 1690393044548,
        };

        let expected_sig = "0x214d507bbdaebba52fa60928f904a8b2df73673e3baba6133d66fe846c7ef70451e82453a6d8db124e7ed6e60fa00d4b7c46e4d96cb2bd61fd81b6e8953cc9d21b";
        assert_eq!(
            sign_typed_data(&usd_send, &wallet)?.to_string(),
            expected_sig
        );
        Ok(())
    }

    #[test]
    fn test_sign_withdraw_from_bridge_action() -> Result<()> {
        let wallet = get_wallet()?;

        let usd_send = Withdraw3 {
            signature_chain_id: 421614,
            hyperliquid_chain: "Testnet".to_string(),
            destination: "0x0D1d9635D0640821d15e323ac8AdADfA9c111414".to_string(),
            amount: "1".to_string(),
            time: 1690393044548,
        };

        let expected_sig = "0xb3172e33d2262dac2b4cb135ce3c167fda55dafa6c62213564ab728b9f9ba76b769a938e9f6d603dae7154c83bf5a4c3ebab81779dc2db25463a3ed663c82ae41c";
        assert_eq!(
            sign_typed_data(&usd_send, &wallet)?.to_string(),
            expected_sig
        );
        Ok(())
    }

    #[test]
    fn test_sign_l1_action_multi_sig() -> Result<()> {
        // Create two test wallets
        let wallet1 = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse::<PrivateKeySigner>()
            .map_err(|e| Error::Wallet(e.to_string()))?;
        let wallet2 = "0000000000000000000000000000000000000000000000000000000000000001"
            .parse::<PrivateKeySigner>()
            .map_err(|e| Error::Wallet(e.to_string()))?;

        let wallets = vec![wallet1, wallet2];
        let connection_id =
            B256::from_str("0xde6c4037798a4434ca03cd05f00e3b803126221375cd1e7eaaaf041768be06eb")
                .map_err(|e| Error::GenericParse(e.to_string()))?;

        // Test mainnet
        let mainnet_sigs = sign_l1_action_multi_sig(&wallets, connection_id, true)?;
        assert_eq!(mainnet_sigs.len(), 2);
        assert_eq!(
            mainnet_sigs[0].to_string(),
            "0xfa8a41f6a3fa728206df80801a83bcbfbab08649cd34d9c0bfba7c7b2f99340f53a00226604567b98a1492803190d65a201d6805e5831b7044f17fd530aec7841c"
        );

        // Test testnet
        let testnet_sigs = sign_l1_action_multi_sig(&wallets, connection_id, false)?;
        assert_eq!(testnet_sigs.len(), 2);
        assert_eq!(
            testnet_sigs[0].to_string(),
            "0x1713c0fc661b792a50e8ffdd59b637b1ed172d9a3aa4d801d9d88646710fb74b33959f4d075a7ccbec9f2374a6da21ffa4448d58d0413a0d335775f680a881431c"
        );

        Ok(())
    }

    #[test]
    fn test_sign_typed_data_multi_sig() -> Result<()> {
        // Create two test wallets
        let wallet1 = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
            .parse::<PrivateKeySigner>()
            .map_err(|e| Error::Wallet(e.to_string()))?;
        let wallet2 = "0000000000000000000000000000000000000000000000000000000000000001"
            .parse::<PrivateKeySigner>()
            .map_err(|e| Error::Wallet(e.to_string()))?;

        let wallets = vec![wallet1, wallet2];

        let usd_send = UsdSend {
            signature_chain_id: 421614,
            hyperliquid_chain: "Testnet".to_string(),
            destination: "0x0D1d9635D0640821d15e323ac8AdADfA9c111414".to_string(),
            amount: "1".to_string(),
            time: 1690393044548,
        };

        let signatures = sign_typed_data_multi_sig(&usd_send, &wallets)?;
        assert_eq!(signatures.len(), 2);

        // First signature should match the single-sig test
        assert_eq!(
            signatures[0].to_string(),
            "0x214d507bbdaebba52fa60928f904a8b2df73673e3baba6133d66fe846c7ef70451e82453a6d8db124e7ed6e60fa00d4b7c46e4d96cb2bd61fd81b6e8953cc9d21b"
        );

        Ok(())
    }

    #[test]
    fn test_multi_sig_with_single_wallet() -> Result<()> {
        // Test that multi-sig works with a single wallet
        let wallet = get_wallet()?;
        let wallets = vec![wallet];
        let connection_id =
            B256::from_str("0xde6c4037798a4434ca03cd05f00e3b803126221375cd1e7eaaaf041768be06eb")
                .map_err(|e| Error::GenericParse(e.to_string()))?;

        let sigs = sign_l1_action_multi_sig(&wallets, connection_id, true)?;
        assert_eq!(sigs.len(), 1);

        // Should match the single-sig result
        assert_eq!(
            sigs[0].to_string(),
            "0xfa8a41f6a3fa728206df80801a83bcbfbab08649cd34d9c0bfba7c7b2f99340f53a00226604567b98a1492803190d65a201d6805e5831b7044f17fd530aec7841c"
        );

        Ok(())
    }

    #[test]
    fn test_sign_multi_sig_l1_action_payload() -> Result<()> {
        let wallet1 = "0x0123456789012345678901234567890123456789012345678901234567890123"
            .parse::<PrivateKeySigner>()
            .map_err(|e| Error::Wallet(e.to_string()))?;
        let wallet2 = "0x0000000000000000000000000000000000000000000000000000000000000001"
            .parse::<PrivateKeySigner>()
            .map_err(|e| Error::Wallet(e.to_string()))?;

        let wallets = vec![wallet1, wallet2];

        let multi_sig_user =
            alloy::primitives::Address::from_str("0x0000000000000000000000000000000000000005")
                .map_err(|e| Error::GenericParse(e.to_string()))?;
        let outer_signer =
            alloy::primitives::Address::from_str("0x0d1d9635d0640821d15e323ac8adadfa9c111414")
                .map_err(|e| Error::GenericParse(e.to_string()))?;

        let action = serde_json::json!({
            "type": "order",
            "orders": [{"a": 4, "b": true, "p": "1100", "s": "0.2", "r": false, "t": {"limit": {"tif": "Gtc"}}}],
            "grouping": "na"
        });

        let nonce = 0u64;

        let signatures_mainnet = sign_multi_sig_l1_action_payload(
            &wallets,
            &action,
            multi_sig_user,
            outer_signer,
            None,
            nonce,
            None,
            true,
        )?;

        assert_eq!(signatures_mainnet.len(), 2);

        let signatures_testnet = sign_multi_sig_l1_action_payload(
            &wallets,
            &action,
            multi_sig_user,
            outer_signer,
            None,
            nonce,
            None,
            false,
        )?;

        assert_eq!(signatures_testnet.len(), 2);

        Ok(())
    }
}
