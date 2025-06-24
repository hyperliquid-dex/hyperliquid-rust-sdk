use alloy::{
    primitives::B256,
    signers::{local::PrivateKeySigner, Signature, SignerSync},
    sol_types::SolStruct,
};

use crate::{errors, prelude::*, signature::agent::l1, Eip712};

pub(crate) fn sign_l1_action(
    signer: &PrivateKeySigner,
    connection_id: B256,
    is_mainnet: bool,
) -> Result<Signature> {
    let source = if is_mainnet { "a" } else { "b" }.to_string();
    sign_typed_data(
        &l1::Agent {
            source,
            connection_id,
        },
        signer,
    )
}

pub(crate) fn sign_typed_data<T: SolStruct + Eip712>(
    payload: &T,
    signer: &PrivateKeySigner,
) -> Result<Signature> {
    // Derive the EIP-712 signing hash.
    let hash = payload.eip712_signing_hash(&payload.domain());

    // Sign the hash asynchronously with the wallet.
    let signature = signer
        .sign_hash_sync(&hash)
        .map_err(|err| errors::Error::SignatureFailure(err.to_string()))?;
    Ok(signature)
}

#[cfg(test)]
mod tests {
    use alloy::primitives::U256;

    use super::*;
    use crate::{Error, UsdSend, Withdraw3};
    use std::str::FromStr;

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

        let expected_mainnet_sig = "0x3b81aff53cb873526a3db775ba6a0f6d71d8ab8e2ca181ff1f830bb81288c3ec59d97ec75d39e66755eef1c1b2d7f2cbaef7cc264fe89176ffb6ef64f7202f631b";
        assert_eq!(
            sign_l1_action(&wallet, connection_id, true)?.to_string(),
            expected_mainnet_sig
        );
        let expected_testnet_sig = "0x9d0c9557cb567641eee5c36aaf4d99a8ebbcf231bb390f86d4ba0507b5042fbd05eebe06b2a69144ee8dea74930e60bab947cfa060bc57ecfbf0c8c83b89b8d41c";
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
            signature_chain_id: U256::from(421614),
            hyperliquid_chain: "Testnet".to_string(),
            destination: "0x0D1d9635D0640821d15e323ac8AdADfA9c111414".to_string(),
            amount: "1".to_string(),
            time: U256::from::<u64>(1690393044548),
        };

        let expected_sig = "0xb5ce88b110d35bdceb3112dcbd7179c164e84080469f362bacc2286d1ace722d3b5dcc63797328e931c30fdb06fcd4f9579583dfaef9babcee8ac62b08bc24131c";
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
            signature_chain_id: U256::from(421614),
            hyperliquid_chain: "Testnet".to_string(),
            destination: "0x0D1d9635D0640821d15e323ac8AdADfA9c111414".to_string(),
            amount: "1".to_string(),
            time: U256::from::<u64>(1690393044548),
        };

        let expected_sig = "0x7e8c10d816c8abd866c2bc45a2d4857304c2a757e98ae34a44e979a5304df2862bcfe34d7eac9b8286118b2872644d3b805e2275a208db0ebda3a633e39c418c1c";
        assert_eq!(
            sign_typed_data(&usd_send, &wallet)?.to_string(),
            expected_sig
        );
        Ok(())
    }
}
