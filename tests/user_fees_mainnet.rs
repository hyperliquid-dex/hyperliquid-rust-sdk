use alloy::primitives::Address;
use hyperliquid_rust_sdk::{BaseUrl, InfoClient};

/// Exercise the live mainnet `/info` endpoint to make sure the freshly added
/// `fee_trial_escrow` field round-trips through deserialization.
#[tokio::test]
async fn user_fees_includes_fee_trial_escrow_on_mainnet() {
    let client = InfoClient::new(None, Some(BaseUrl::Mainnet))
        .await
        .expect("create mainnet info client");
    let user: Address = "0xc64cc00b46101bd40aa1c3121195e85c0b0918d8"
        .parse()
        .expect("parse hard-coded mainnet address");

    let response = client
        .user_fees(user)
        .await
        .expect("fetch mainnet user fees");

    assert!(
        !response.fee_trial_escrow.is_empty(),
        "expected `fee_trial_escrow` to be present in the mainnet response"
    );
    assert!(
        response.fee_trial_escrow.parse::<f64>().is_ok(),
        "`fee_trial_escrow` should be numeric but was {}",
        response.fee_trial_escrow
    );
}
