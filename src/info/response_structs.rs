use crate::{
    info::{AssetPosition, Level},
    DailyUserVlm, Delta, FeeSchedule, Leverage, MarginSummary, OrderInfo, Referrer, ReferrerState,
    UserTokenBalance,
};
use alloy::primitives::Address;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserStateResponse {
    pub asset_positions: Vec<AssetPosition>,
    pub cross_margin_summary: MarginSummary,
    pub margin_summary: MarginSummary,
    pub withdrawable: String,
}

#[derive(Deserialize, Debug)]
pub struct UserTokenBalanceResponse {
    pub balances: Vec<UserTokenBalance>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActiveAssetDataResponse {
    pub user: Address,
    pub coin: String,
    pub leverage: Leverage,
    pub max_trade_szs: Vec<String>,
    pub available_to_trade: Vec<String>,
    pub mark_px: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFeesResponse {
    pub active_referral_discount: String,
    pub daily_user_vlm: Vec<DailyUserVlm>,
    pub fee_schedule: FeeSchedule,
    pub user_add_rate: String,
    pub user_cross_rate: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrdersResponse {
    pub coin: String,
    pub limit_px: String,
    pub oid: u64,
    pub side: String,
    pub sz: String,
    pub timestamp: u64,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFillsResponse {
    pub closed_pnl: String,
    pub coin: String,
    pub crossed: bool,
    pub dir: String,
    pub hash: String,
    pub oid: u64,
    pub px: String,
    pub side: String,
    pub start_position: String,
    pub sz: String,
    pub time: u64,
    pub fee: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FundingHistoryResponse {
    pub coin: String,
    pub funding_rate: String,
    pub premium: String,
    pub time: u64,
}

#[derive(Deserialize, Debug)]
pub struct UserFundingResponse {
    pub time: u64,
    pub hash: String,
    pub delta: Delta,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct L2SnapshotResponse {
    pub coin: String,
    pub levels: Vec<Vec<Level>>,
    pub time: u64,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecentTradesResponse {
    pub coin: String,
    pub side: String,
    pub px: String,
    pub sz: String,
    pub time: u64,
    pub hash: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct CandlesSnapshotResponse {
    #[serde(rename = "t")]
    pub time_open: u64,
    #[serde(rename = "T")]
    pub time_close: u64,
    #[serde(rename = "s")]
    pub coin: String,
    #[serde(rename = "i")]
    pub candle_interval: String,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "v")]
    pub vlm: String,
    #[serde(rename = "n")]
    pub num_trades: u64,
}

#[derive(Deserialize, Debug)]
pub struct OrderStatusResponse {
    pub status: String,
    /// `None` if the order is not found
    #[serde(default)]
    pub order: Option<OrderInfo>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReferralResponse {
    pub referred_by: Option<Referrer>,
    pub cum_vlm: String,
    pub unclaimed_rewards: String,
    pub claimed_rewards: String,
    pub referrer_state: ReferrerState,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserRateLimitResponse {
    pub cum_vlm: String,
    pub n_requests_used: u32,
    pub n_requests_cap: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PerpDexLimitsResponse {
    pub total_oi_cap: String,
    pub oi_sz_cap_per_perp: String,
    pub max_transfer_ntl: String,
    pub coin_to_oi_cap: Vec<[String; 2]>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerpDexInfo {
    pub name: String,
    pub full_name: String,
    pub deployer: String,
    pub oracle_updater: Option<String>,
    pub fee_recipient: Option<String>,
    pub asset_to_streaming_oi_cap: Vec<[String; 2]>,
}

/// Response from perpDexs endpoint
/// API returns [null, {...}, {...}, ...] format
/// First element is null, followed by PerpDexInfo objects
#[derive(Debug)]
pub struct PerpDexsResponse(pub Vec<PerpDexInfo>);

impl<'de> serde::Deserialize<'de> for PerpDexsResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let values: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;

        // Skip the first element (null) and deserialize the rest
        let perp_dexs: Result<Vec<PerpDexInfo>, _> = values
            .into_iter()
            .skip(1)
            .map(|v| serde_json::from_value(v).map_err(serde::de::Error::custom))
            .collect();

        Ok(PerpDexsResponse(perp_dexs?))
    }
}

impl PerpDexsResponse {
    /// Get all perp dex info as a slice
    pub fn perp_dexs(&self) -> &[PerpDexInfo] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perp_dexs_response_parsing_single() {
        // Test parsing perpDexs response from mainnet (single perp dex)
        let json = r#"[null,{"name":"xyz","fullName":"XYZ","deployer":"0x88806a71d74ad0a510b350545c9ae490912f0888","oracleUpdater":"0x1234567890545d1df9ee64b35fdd16966e08acec","feeRecipient":"0x79c0650064b10f73649b7b64c5ebf0b319606140","assetToStreamingOiCap":[["xyz:XYZ100","100000000.0"]]}]"#;

        let response: PerpDexsResponse = serde_json::from_str(json).unwrap();

        // Get perp dexs using helper method
        let perp_dexs = response.perp_dexs();
        assert_eq!(perp_dexs.len(), 1);

        let xyz = &perp_dexs[0];
        assert_eq!(xyz.name, "xyz");
        assert_eq!(xyz.full_name, "XYZ");
        assert_eq!(
            xyz.deployer,
            "0x88806a71d74ad0a510b350545c9ae490912f0888"
        );
        assert_eq!(
            xyz.oracle_updater,
            Some("0x1234567890545d1df9ee64b35fdd16966e08acec".to_string())
        );
        assert_eq!(
            xyz.fee_recipient,
            Some("0x79c0650064b10f73649b7b64c5ebf0b319606140".to_string())
        );
        assert_eq!(xyz.asset_to_streaming_oi_cap.len(), 1);
        assert_eq!(xyz.asset_to_streaming_oi_cap[0][0], "xyz:XYZ100");
        assert_eq!(xyz.asset_to_streaming_oi_cap[0][1], "100000000.0");
    }

    #[test]
    fn test_perp_dexs_response_parsing_multiple() {
        // Test parsing perpDexs response with multiple perp dexs (testnet format)
        let json = r#"[null,{"name":"test","fullName":"test dex","deployer":"0x5e89b26d8d66da9888c835c9bfcc2aa51813e152","oracleUpdater":null,"feeRecipient":null,"assetToStreamingOiCap":[]},{"name":"xyz","fullName":"XYZ","deployer":"0x88806a71d74ad0a510b350545c9ae490912f0888","oracleUpdater":"0x1234567890545d1df9ee64b35fdd16966e08acec","feeRecipient":"0x79c0650064b10f73649b7b64c5ebf0b319606140","assetToStreamingOiCap":[["xyz:XYZ100","100000000.0"]]}]"#;

        let response: PerpDexsResponse = serde_json::from_str(json).unwrap();

        // Get perp dexs using helper method
        let perp_dexs = response.perp_dexs();
        assert_eq!(perp_dexs.len(), 2);

        assert_eq!(perp_dexs[0].name, "test");
        assert_eq!(perp_dexs[1].name, "xyz");
    }

    #[test]
    fn test_perp_dex_info_clone() {
        // Test that PerpDexInfo is cloneable
        let perp_dex = PerpDexInfo {
            name: "test".to_string(),
            full_name: "Test".to_string(),
            deployer: "0x123".to_string(),
            oracle_updater: Some("0x456".to_string()),
            fee_recipient: Some("0x789".to_string()),
            asset_to_streaming_oi_cap: vec![["test:ABC".to_string(), "1000000.0".to_string()]],
        };

        let cloned = perp_dex.clone();
        assert_eq!(perp_dex.name, cloned.name);
        assert_eq!(perp_dex.full_name, cloned.full_name);
        assert_eq!(perp_dex.oracle_updater, cloned.oracle_updater);
    }

    #[test]
    fn test_perp_dexs_response_testnet_full() {
        // Test with a subset of actual testnet response
        let json = r#"[null,{"name":"test","fullName":"test dex","deployer":"0x5e89b26d8d66da9888c835c9bfcc2aa51813e152","oracleUpdater":null,"feeRecipient":null,"assetToStreamingOiCap":[]},{"name":"vntls","fullName":"Ventuals","deployer":"0xc65008a70f511ae0407d26022ff1516422acea94","oracleUpdater":null,"feeRecipient":null,"assetToStreamingOiCap":[["vntls:vANDRL","450000.0"],["vntls:vANTHRPC","600000.0"]]},{"name":"xyz","fullName":"XYZ","deployer":"0x7770132f86dd4002d018edd7d348f4a0e4340777","oracleUpdater":"0x0000000000c7153b8c085618fb91cb1214092201","feeRecipient":"0x23955d36db94d0f25216045f2bcc34dc5e0f326a","assetToStreamingOiCap":[["xyz:XYZ100","30000000.0"]]}]"#;

        let response: PerpDexsResponse = serde_json::from_str(json).unwrap();
        let perp_dexs = response.perp_dexs();

        assert_eq!(perp_dexs.len(), 3);

        // Test first dex with null values
        assert_eq!(perp_dexs[0].name, "test");
        assert_eq!(perp_dexs[0].oracle_updater, None);
        assert_eq!(perp_dexs[0].fee_recipient, None);
        assert_eq!(perp_dexs[0].asset_to_streaming_oi_cap.len(), 0);

        // Test second dex with assets
        assert_eq!(perp_dexs[1].name, "vntls");
        assert_eq!(perp_dexs[1].asset_to_streaming_oi_cap.len(), 2);
        assert_eq!(perp_dexs[1].asset_to_streaming_oi_cap[0][0], "vntls:vANDRL");

        // Test third dex with all fields populated
        assert_eq!(perp_dexs[2].name, "xyz");
        assert!(perp_dexs[2].oracle_updater.is_some());
        assert!(perp_dexs[2].fee_recipient.is_some());
    }
}
