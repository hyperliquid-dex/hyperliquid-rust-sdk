use alloy::{
    dyn_abi::Eip712Domain,
    primitives::{Address, B256, U256},
    signers::{local::PrivateKeySigner, Signer, SignerSync},
    sol_types::{SolStruct, SolType, SolValue},
};

use crate::{prelude::*, Error};
use hex;

#[derive(Debug, Clone)]
pub struct SignatureBytes(pub [u8; 65]);

impl ToString for SignatureBytes {
    fn to_string(&self) -> String {
        hex::encode(self.0)
    }
}

pub(crate) mod domain {
    use super::*;

    alloy::sol! {
        #[derive(Debug)]
        struct Domain {
            string name;
            string version;
            uint256 chainId;
            address verifyingContract;
            bytes32 salt;
        }
    }

    impl Domain {
        pub fn new() -> Self {
            Self {
                name: "HyperLiquid".into(),
                version: "1".into(),
                chainId: U256::from(421614u64),
                verifyingContract: Address::ZERO,
                salt: B256::ZERO,
            }
        }
    }
}

pub(crate) async fn sign_typed_data<T: SolStruct>(
    payload: &T,
    wallet: &PrivateKeySigner,
) -> Result<SignatureBytes> {
    let domain = Eip712Domain {
        name: Some("HyperLiquid".into()),
        version: Some("1".into()),
        chain_id: Some(U256::from(421614u64)),
        verifying_contract: None,
        salt: None,
    };

    let hash = payload.eip712_signing_hash(&domain);
    let signature = wallet
        .sign_hash(&hash)
        .await
        .map_err(|e| Error::SignatureFailure(e.to_string()))?;

    Ok(SignatureBytes(signature.as_bytes()))
}

pub(crate) async fn sign_l1_action(
    hash: B256,
    wallet: &PrivateKeySigner,
) -> Result<SignatureBytes> {
    let signature = wallet
        .sign_hash_sync(&hash)
        .map_err(|e| Error::SignatureFailure(e.to_string()))?;

    Ok(SignatureBytes(signature.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UsdSend, Withdraw3};
    use std::str::FromStr;

    fn get_wallet() -> Result<PrivateKeySigner> {
        let priv_key = "e908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e";
        priv_key
            .parse::<PrivateKeySigner>()
            .map_err(|e| Error::PrivateKeyParse(e.to_string()))
    }

    #[tokio::test]
    async fn test_sign_l1_action() -> Result<()> {
        let wallet = get_wallet()?;
        let connection_id =
            B256::from_str("0xde6c4037798a4434ca03cd05f00e3b803126221375cd1e7eaaaf041768be06eb")
                .map_err(|e| Error::GenericParse(e.to_string()))?;

        let expected_mainnet_sig = "f88f65c5bf58dac71b4f0fab0e16da760c29e766a1c0970ed4738a9c3c5b1d0d72b3bfc5cbf0e01bd31ffab7c9cfc7f54b0c80532444da8b8ff710074ee9fec31b";
        assert_eq!(
            sign_l1_action(connection_id, &wallet).await?.to_string(),
            expected_mainnet_sig
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sign_usd_transfer_action() -> Result<()> {
        let wallet = get_wallet()?;

        let usd_send = UsdSend {
            signatureChainId: U256::from(421614u64),
            hyperliquidChain: "Testnet".to_string(),
            destination: Address::from_str("0x0D1d9635D0640821d15e323ac8AdADfA9c111414")
                .map_err(|e| Error::GenericParse(e.to_string()))?,
            amount: U256::from(1u64),
            time: U256::from(1690393044548u64),
        };

        let expected_sig = "c1408a72b086344c3c0837847f60857486c81f9f69c4d05e2cb2a8de027c7bdd2d611524585e82e76eeee31c75ad332bd5d89a91f1cbfa20d039a4e2a77e30bd1c";
        assert_eq!(
            sign_typed_data(&usd_send, &wallet).await?.to_string(),
            expected_sig
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_sign_withdraw_from_bridge_action() -> Result<()> {
        let wallet = get_wallet()?;

        let usd_send = Withdraw3 {
            signatureChainId: U256::from(421614u64),
            hyperliquidChain: "Testnet".to_string(),
            destination: Address::from_str("0x0D1d9635D0640821d15e323ac8AdADfA9c111414")
                .map_err(|e| Error::GenericParse(e.to_string()))?,
            amount: U256::from(1u64),
            time: U256::from(1690393044548u64),
        };

        let expected_sig = "a9c18df3b6321fbcaf685f526a0d9356f5562dfa608f2cddea9081921815a4337050a25f2503bcfa9cf2b724178fa4c52ce705988cf7b71d6c9a1193fdc0afd11c";
        assert_eq!(
            sign_typed_data(&usd_send, &wallet).await?.to_string(),
            expected_sig
        );
        Ok(())
    }
}
