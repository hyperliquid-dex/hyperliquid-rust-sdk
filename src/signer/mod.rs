use crate::prelude::*;
use crate::proxy_digest::Sha256Proxy;
use crate::Error;
use async_trait::async_trait;
use ethers::core::k256::{elliptic_curve::FieldBytes, Secp256k1};
use ethers::types::{Address, Signature, H256, U256};
use ethers::utils::hex::ToHexExt;
use privy::Privy;
use std::sync::Arc;

#[async_trait]
pub trait Signer: Send + Sync + std::fmt::Debug {
    async fn secp256k1_sign(&self, message: H256) -> Result<Signature>;
    fn address(&self) -> Address;
}

#[async_trait]
impl<T: Signer + ?Sized> Signer for Arc<T> {
    async fn secp256k1_sign(&self, message: H256) -> Result<Signature> {
        (**self).secp256k1_sign(message).await
    }
    fn address(&self) -> Address {
        (**self).address()
    }
}

#[derive(Debug)]
pub struct PrivySigner {
    pub privy: Privy,
    pub wallet_id: String,
    pub address: String,
}

impl PrivySigner {
    pub fn new(privy: Privy, wallet_id: String, address: String) -> Self {
        Self {
            privy,
            wallet_id,
            address,
        }
    }
}

pub fn signature_string_to_ethers_signature(signature: String) -> Result<Signature> {
    // Privy returns a hex string like "0x..." that is 65 bytes (130 hex chars + 2 for "0x")
    let signature = signature.strip_prefix("0x").unwrap_or(&signature);

    // The signature is in format: r (32 bytes) + s (32 bytes) + v (1 byte)
    let r = U256::from_str_radix(&signature[0..64], 16)
        .map_err(|e| Error::SignatureFailure(format!("Failed to parse r: {}", e)))?;
    let s = U256::from_str_radix(&signature[64..128], 16)
        .map_err(|e| Error::SignatureFailure(format!("Failed to parse s: {}", e)))?;
    let v = u64::from_str_radix(&signature[128..130], 16)
        .map_err(|e| Error::SignatureFailure(format!("Failed to parse v: {}", e)))?;

    Ok(Signature { r, s, v })
}

#[async_trait]
impl Signer for PrivySigner {
    fn address(&self) -> Address {
        self.address.parse().unwrap()
    }

    async fn secp256k1_sign(&self, message: H256) -> Result<Signature> {
        log::debug!("Signing message: {}", message.to_string());
        let signature = self
            .privy
            .secp256k1_sign(
                self.wallet_id.clone(),
                format!("0x{}", message.as_bytes().encode_hex()),
            )
            .await
            .map_err(|e| Error::SignatureFailure(e.to_string()))?;

        signature_string_to_ethers_signature(signature)
    }
}

#[async_trait]
impl Signer for ethers::signers::LocalWallet {
    async fn secp256k1_sign(&self, message: H256) -> Result<Signature> {
        let (sig, rec_id) = self
            .signer()
            .sign_digest_recoverable(Sha256Proxy::from(message))
            .map_err(|e| Error::SignatureFailure(e.to_string()))?;

        let v = u8::from(rec_id) as u64 + 27;

        let r_bytes: FieldBytes<Secp256k1> = sig.r().into();
        let s_bytes: FieldBytes<Secp256k1> = sig.s().into();
        let r = U256::from_big_endian(r_bytes.as_slice());
        let s = U256::from_big_endian(s_bytes.as_slice());

        Ok(Signature { r, s, v })
    }

    fn address(&self) -> Address {
        ethers::signers::Signer::address(self)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethers::utils::hex::ToHexExt;
    use privy::config::PrivyConfig;

    use super::*;

    const TEST_WALLET_ID: &str = "k0pq0k5an1fvo35m5gm3wn8d";
    const TEST_ADDRESS: &str = "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770";

    #[tokio::test]
    #[ignore = "this requires a private key thats also a privy wallet and PRIVY_* env vars"]
    async fn test_secp256k1_sign_convergence() {
        dotenv::dotenv().ok();
        let privy_signer = Arc::new(PrivySigner::new(
            Privy::new(PrivyConfig::from_env().unwrap()),
            TEST_WALLET_ID.to_string(),
            TEST_ADDRESS.to_string(),
        ));
        let private_key = std::env::var("PRIVATE_KEY").unwrap();
        let local_wallet: Arc<ethers::signers::LocalWallet> =
            Arc::new(private_key.parse().unwrap());

        let message =
            H256::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
                .unwrap();

        let privy_signature = privy_signer.secp256k1_sign(message).await.unwrap();
        let local_signature = local_wallet.secp256k1_sign(message).await.unwrap();

        assert_eq!(privy_signature, local_signature);
    }

    #[test]
    fn test_hash_manipulation() {
        let message_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let message = H256::from_str(message_str).unwrap();
        H256::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
            .unwrap();

        let message_hex = format!("0x{}", message.as_bytes().encode_hex());

        assert_eq!(message_hex, message_str);
    }
}
