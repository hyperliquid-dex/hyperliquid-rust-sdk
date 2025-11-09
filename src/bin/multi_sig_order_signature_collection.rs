/// Example: Multi-sig order placement with signature collection workflow
///
/// This demonstrates how to collect signatures for L1 actions (orders)
/// where each signer creates their signature independently.
///
/// Usage:
///   cargo run --bin multi_sig_order_signature_collection
use alloy::signers::{local::PrivateKeySigner, Signature};
use hyperliquid_rust_sdk::sign_multi_sig_l1_action_single;
use log::info;
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    env_logger::init();

    info!("=== Multi-Sig Order Signature Collection Demo ===\n");

    demonstrate_order_signature_collection()?;

    Ok(())
}

fn demonstrate_order_signature_collection() -> Result<()> {
    // Setup: Define the multi-sig parameters
    let multi_sig_user =
        alloy::primitives::Address::from_str("0x0000000000000000000000000000000000000005")?;
    let outer_signer =
        alloy::primitives::Address::from_str("0x0d1d9635d0640821d15e323ac8adadfa9c111414")?;
    let nonce = 1234567890u64;

    info!("Multi-sig parameters:");
    info!("  Multi-sig user: {}", multi_sig_user);
    info!("  Outer signer: {}", outer_signer);
    info!("  Nonce: {}\n", nonce);

    // Create the order action
    // All signers must create the exact same action
    let action = serde_json::json!({
        "type": "order",
        "orders": [{
            "a": 0,          // asset index (0 = BTC)
            "b": true,       // is_buy
            "p": "30000",    // limit price
            "s": "0.1",      // size
            "r": false,      // reduce_only
            "t": {"limit": {"tif": "Gtc"}}
        }],
        "grouping": "na"
    });

    info!("Order action:");
    info!("{}\n", serde_json::to_string_pretty(&action)?);

    // Step 1: Each signer creates their signature independently
    info!("Step 1: Each signer creates their signature\n");

    let signer1_wallet = "0xe908f86dbb4d55ac876378565aafeabc187f6690f046459397b17d9b9a19688e"
        .parse::<PrivateKeySigner>()?;
    let signer2_wallet = "0x0000000000000000000000000000000000000000000000000000000000000001"
        .parse::<PrivateKeySigner>()?;

    info!("Signer 1 address: {}", signer1_wallet.address());
    info!("Signer 2 address: {}", signer2_wallet.address());

    // Signer 1 signs the L1 action
    let sig1 = sign_multi_sig_l1_action_single(
        &signer1_wallet,
        &action,
        multi_sig_user,
        outer_signer,
        None, // vault_address
        nonce,
        None,  // expires_after
        false, // is_mainnet = false (testnet)
    )?;
    info!("\nSigner 1 signature: {}", sig1);

    // Signer 2 signs the L1 action
    let sig2 = sign_multi_sig_l1_action_single(
        &signer2_wallet,
        &action,
        multi_sig_user,
        outer_signer,
        None,
        nonce,
        None,
        false,
    )?;
    info!("Signer 2 signature: {}", sig2);

    // Step 2: Signatures are serialized and transmitted
    info!("\nStep 2: Signatures are exported for transmission\n");

    let sig1_string = sig1.to_string();
    let sig2_string = sig2.to_string();

    info!("Sig 1 exported: {}", sig1_string);
    info!("Sig 2 exported: {}", sig2_string);

    // Step 3: Submitter collects and imports signatures
    info!("\nStep 3: Submitter collects signatures\n");

    let collected_signatures = [sig1_string, sig2_string];
    info!("Collected {} signatures", collected_signatures.len());

    // Import signatures
    let signatures: Vec<Signature> = collected_signatures
        .iter()
        .map(|s| s.parse().expect("Failed to import signature"))
        .collect();

    info!("Successfully imported {} signatures", signatures.len());

    // Step 4: Show how to submit (commented out to avoid actual submission)
    info!("\nStep 4: Submit order (example - not executed)\n");

    info!("To submit, the outer signer would run:");
    info!("```rust");
    info!("let submitter_wallet = \"YOUR_KEY\".parse::<PrivateKeySigner>()?;");
    info!("let sdk = ExchangeClient::new(submitter_wallet, Some(BaseUrl::Testnet), None).await?;");
    info!("");
    info!("let order = ClientOrderRequest {{");
    info!("    asset: \"BTC\".to_string(),");
    info!("    is_buy: true,");
    info!("    reduce_only: false,");
    info!("    limit_px: 30000.0,");
    info!("    sz: 0.1,");
    info!("    order_type: ClientOrderType::Limit(ClientLimit {{");
    info!("        tif: \"Gtc\".to_string(),");
    info!("    }}),");
    info!("    cloid: None,");
    info!("}};");
    info!("");
    info!("sdk.multi_sig_order_with_signatures(");
    info!("    multi_sig_user,");
    info!("    order,");
    info!("    signatures,");
    info!(").await?;");
    info!("```");

    info!("\n=== Demo Complete ===");
    info!("\nKey differences for L1 actions (orders):");
    info!("1. Use sign_multi_sig_l1_action_single instead of sign_multi_sig_user_signed_action_single");
    info!("2. Sign the JSON action directly (type + orders/cancels/etc)");
    info!("3. Must specify vault_address and expires_after parameters");
    info!("4. Network parameter (is_mainnet) affects the signature");

    Ok(())
}
