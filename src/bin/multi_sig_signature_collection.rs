/// Example: Multi-sig USDC transfer with signature collection workflow
///
/// This demonstrates the recommended approach where signatures are collected
/// from different parties independently, rather than having all private keys
/// in one place.
///
/// Usage:
///   cargo run --bin multi_sig_signature_collection
use alloy::signers::{local::PrivateKeySigner, Signature};
use hyperliquid_rust_sdk::{sign_multi_sig_user_signed_action_single, SendAsset};
use log::info;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    env_logger::init();

    info!("=== Multi-Sig Signature Collection Demo ===\n");

    // Simulate the workflow
    demonstrate_signature_collection()?;

    Ok(())
}

fn demonstrate_signature_collection() -> Result<()> {
    // Setup: Define the multi-sig parameters
    // In a real scenario, these would be coordinated among all parties
    let multi_sig_user =
        alloy::primitives::Address::from_str("0x0000000000000000000000000000000000000005")?;
    let outer_signer =
        alloy::primitives::Address::from_str("0x0d1d9635d0640821d15e323ac8adadfa9c111414")?;
    let destination = "0x1234567890123456789012345678901234567890";
    let amount = "100";
    let nonce = 1234567890u64;

    info!("Multi-sig parameters:");
    info!("  Multi-sig user: {}", multi_sig_user);
    info!("  Outer signer: {}", outer_signer);
    info!("  Destination: {}", destination);
    info!("  Amount: {}", amount);
    info!("  Nonce: {}\n", nonce);

    // Step 1: Each signer creates their signature independently
    info!("Step 1: Each signer creates their signature\n");

    let signer1_wallet = "0xe908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse::<PrivateKeySigner>()?;
    let signer2_wallet = "0x0000000000000000000000000000000000000000000000000000000000000001"
        .parse::<PrivateKeySigner>()?;

    info!("Signer 1 address: {}", signer1_wallet.address());
    info!("Signer 2 address: {}", signer2_wallet.address());

    // Both signers create the same SendAsset action
    let send_asset = create_send_asset(multi_sig_user, outer_signer, destination, amount, nonce);

    // Signer 1 signs
    let sig1 = sign_multi_sig_user_signed_action_single(&signer1_wallet, &send_asset)?;
    info!("\nSigner 1 signature: {}", sig1);

    // Signer 2 signs
    let sig2 = sign_multi_sig_user_signed_action_single(&signer2_wallet, &send_asset)?;
    info!("Signer 2 signature: {}", sig2);

    // Step 2: Signatures are serialized and transmitted
    info!("\nStep 2: Signatures are exported for transmission\n");

    let sig1_string = export_signature(&sig1);
    let sig2_string = export_signature(&sig2);

    info!("Sig 1 exported: {}", sig1_string);
    info!("Sig 2 exported: {}", sig2_string);

    // Step 3: Submitter collects and imports signatures
    info!("\nStep 3: Submitter collects signatures\n");

    let collected_signatures = [sig1_string, sig2_string];
    info!("Collected {} signatures", collected_signatures.len());

    // Import signatures
    let signatures: Vec<Signature> = collected_signatures
        .iter()
        .map(|s| import_signature(s).expect("Failed to import signature"))
        .collect();

    info!("Successfully imported {} signatures", signatures.len());

    // Step 4: Show how to submit (commented out to avoid actual submission)
    info!("\nStep 4: Submit transaction (example - not executed)\n");

    info!("To submit, the outer signer would run:");
    info!("```rust");
    info!("let submitter_wallet = \"YOUR_KEY\".parse::<PrivateKeySigner>()?;");
    info!("let sdk = ExchangeClient::new(submitter_wallet, Some(BaseUrl::Testnet), None).await?;");
    info!("");
    info!("sdk.multi_sig_usdc_transfer_with_signatures(");
    info!("    multi_sig_user,");
    info!("    \"{}\",", amount);
    info!("    \"{}\",", destination);
    info!("    signatures,");
    info!(").await?;");
    info!("```");

    info!("\n=== Demo Complete ===");
    info!("\nKey takeaways:");
    info!("1. Each signer creates their signature independently");
    info!("2. Signatures can be serialized as hex strings for transmission");
    info!("3. The submitter collects and combines signatures");
    info!("4. All signers must sign identical action parameters");
    info!("5. The outer_signer (submitter) doesn't need to be a multi-sig participant");

    Ok(())
}

/// Create a SendAsset action - must be identical for all signers
fn create_send_asset(
    multi_sig_user: alloy::primitives::Address,
    outer_signer: alloy::primitives::Address,
    destination: &str,
    amount: &str,
    nonce: u64,
) -> SendAsset {
    SendAsset {
        signature_chain_id: 421614,
        hyperliquid_chain: "Testnet".to_string(),
        destination: destination.to_string(),
        source_dex: "".to_string(),
        destination_dex: "".to_string(),
        token: "USDC".to_string(),
        amount: amount.to_string(),
        from_sub_account: "".to_string(),
        nonce,
        payload_multi_sig_user: Some(format!("{:#x}", multi_sig_user).to_lowercase()),
        outer_signer: Some(format!("{:#x}", outer_signer).to_lowercase()),
    }
}

/// Export a signature as a hex string
fn export_signature(sig: &Signature) -> String {
    sig.to_string()
}

/// Import a signature from a hex string
fn import_signature(sig_str: &str) -> Result<Signature> {
    sig_str
        .parse()
        .map_err(|e| format!("Failed to parse signature: {:?}", e).into())
}
