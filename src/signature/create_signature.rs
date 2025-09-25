use alloy::{
    primitives::{keccak256, Address, B256},
    signers::{local::PrivateKeySigner, Signature, SignerSync},
};

use serde::Serialize;

use crate::{
    eip712::Eip712,
    prelude::*,
    signature::agent::l1,
    ApproveAgent,
    ApproveBuilderFee,
    Error,
    SpotSend,
    UsdSend,
    Withdraw3,
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

fn connection_id_from_action<T: Serialize>(
    action: &T,
    nonce: u64,
    vault_address: Option<Address>,
    expires_after: Option<u64>,
) -> Result<B256> {
    let mut bytes = rmp_serde::to_vec_named(action).map_err(|e| Error::RmpParse(e.to_string()))?;
    bytes.extend(nonce.to_be_bytes());
    if let Some(address) = vault_address {
        bytes.push(1);
        bytes.extend(address);
    } else {
        bytes.push(0);
    }
    if let Some(expiration) = expires_after {
        bytes.push(0);
        bytes.extend(expiration.to_be_bytes());
    }
    Ok(keccak256(bytes))
}

pub(crate) fn sign_l1_action_v2<T: Serialize>(
    wallet: &PrivateKeySigner,
    action: &T,
    vault_address: Option<Address>,
    nonce: u64,
    expires_after: Option<u64>,
    is_mainnet: bool,
) -> Result<Signature> {
    let connection_id = connection_id_from_action(action, nonce, vault_address, expires_after)?;
    sign_l1_action(wallet, connection_id, is_mainnet)
}

pub(crate) trait UserSignedPayload: Eip712 {
    fn set_signature_chain_id(&mut self, chain_id: u64);
    fn set_hyperliquid_chain(&mut self, hyperliquid_chain: &str);
}

impl UserSignedPayload for UsdSend {
    fn set_signature_chain_id(&mut self, chain_id: u64) {
        self.signature_chain_id = chain_id;
    }

    fn set_hyperliquid_chain(&mut self, hyperliquid_chain: &str) {
        self.hyperliquid_chain = hyperliquid_chain.to_string();
    }
}

impl UserSignedPayload for Withdraw3 {
    fn set_signature_chain_id(&mut self, chain_id: u64) {
        self.signature_chain_id = chain_id;
    }

    fn set_hyperliquid_chain(&mut self, hyperliquid_chain: &str) {
        self.hyperliquid_chain = hyperliquid_chain.to_string();
    }
}

impl UserSignedPayload for SpotSend {
    fn set_signature_chain_id(&mut self, chain_id: u64) {
        self.signature_chain_id = chain_id;
    }

    fn set_hyperliquid_chain(&mut self, hyperliquid_chain: &str) {
        self.hyperliquid_chain = hyperliquid_chain.to_string();
    }
}

impl UserSignedPayload for ApproveAgent {
    fn set_signature_chain_id(&mut self, chain_id: u64) {
        self.signature_chain_id = chain_id;
    }

    fn set_hyperliquid_chain(&mut self, hyperliquid_chain: &str) {
        self.hyperliquid_chain = hyperliquid_chain.to_string();
    }
}

impl UserSignedPayload for ApproveBuilderFee {
    fn set_signature_chain_id(&mut self, chain_id: u64) {
        self.signature_chain_id = chain_id;
    }

    fn set_hyperliquid_chain(&mut self, hyperliquid_chain: &str) {
        self.hyperliquid_chain = hyperliquid_chain.to_string();
    }
}

pub(crate) fn sign_user_signed_action<T: UserSignedPayload>(
    wallet: &PrivateKeySigner,
    action: &mut T,
    signature_chain_id: u64,
    is_mainnet: bool,
) -> Result<Signature> {
    action.set_signature_chain_id(signature_chain_id);
    action.set_hyperliquid_chain(if is_mainnet { "Mainnet" } else { "Testnet" });
    sign_typed_data(action, wallet)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::primitives::address;

    use super::*;
    use crate::{Actions, ApproveBuilderFee, ClaimRewards, Limit, Order, OrderRequest};

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
    fn test_sign_l1_action_v2() -> Result<()> {
        let wallet = get_wallet()?;
        let action = Actions::ClaimRewards(ClaimRewards {});
        let nonce = 1583838;

        let signature = sign_l1_action_v2(&wallet, &action, None, nonce, None, true)?;
        assert_eq!(
            signature.to_string(),
            "0xe13542800ba5ec821153401e1cafac484d1f861adbbb86c00b580ec2560c153248b8d9f0e004ecc86959c07d44b591861ebab2167b54651a81367e2c3d472d4e1c"
        );

        let signature = sign_l1_action_v2(&wallet, &action, None, nonce, None, false)?;
        assert_eq!(
            signature.to_string(),
            "0x16de9b346ddd8e200492a2db45ec9104dcdfc7fbfdbcd85890a6063bdd56df2c44846714c261a431de7095ad52e07143346eb26d9e66c6aed4674f120a1048131c"
        );

        Ok(())
    }

    #[test]
    fn test_sign_l1_action_v2_with_order_action() -> Result<()> {
        let wallet = get_wallet()?;
        let action = Actions::Order(crate::BulkOrder {
            orders: vec![OrderRequest {
                asset: 0,
                is_buy: false,
                limit_px: "102736".to_string(),
                sz: "0.00009".to_string(),
                reduce_only: true,
                order_type: Order::Limit(Limit {
                    tif: "FrontendMarket".to_string(),
                }),
                cloid: None,
            }],
            grouping: "na".to_string(),
            builder: None,
        });
        let expires_after = Some(1_758_784_189_472u64);
        let nonce = 1_758_784_176_087u64;

        let signature = sign_l1_action_v2(&wallet, &action, None, nonce, expires_after, true)?;
        assert_eq!(
            signature.to_string(),
            "0xd141ee5f8da9bbf4eda5719c79c95c2f3e418efb6cf54398ad5024e71c2cc6612b8bdc1ae41a1284e02671fe6ec19ecc39a66f6cf82c23409651b425d6c5ec971b"
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
    fn test_sign_user_signed_action() -> Result<()> {
        let wallet = get_wallet()?;

        let builder = address!("0x1234567890123456789012345678901234567890");
        let mut approve_builder_fee = ApproveBuilderFee {
            signature_chain_id: 0,
            hyperliquid_chain: String::new(),
            builder,
            max_fee_rate: "0.001%".to_string(),
            nonce: 1583838,
        };

        let signature = sign_user_signed_action(&wallet, &mut approve_builder_fee, 421614, true)?;
        assert_eq!(approve_builder_fee.signature_chain_id, 421614);
        assert_eq!(approve_builder_fee.hyperliquid_chain, "Mainnet");
        assert_eq!(
            signature.to_string(),
            "0x343c9078af7c3d6683abefd0ca3b2960de5b669b716863e6dc49090853a4a3cd6c016301239461091a8ca3ea5ac783362526c4d9e9e624ffc563aea93d6ac2391b"
        );

        let mut approve_builder_fee = ApproveBuilderFee {
            signature_chain_id: 0,
            hyperliquid_chain: String::new(),
            builder,
            max_fee_rate: "0.001%".to_string(),
            nonce: 1583838,
        };

        let signature =
            sign_user_signed_action(&wallet, &mut approve_builder_fee, 421614, false)?;
        assert_eq!(approve_builder_fee.signature_chain_id, 421614);
        assert_eq!(approve_builder_fee.hyperliquid_chain, "Testnet");
        assert_eq!(
            signature.to_string(),
            "0x2ada43eeebeba9cfe13faf95aa84e5b8c4885c3a07cbf4536f2df5edd340d4eb1ed0e24f60a80d199a842258d5fa737a18d486f7d4e656268b434d226f2811d71c"
        );

        Ok(())
    }
}
