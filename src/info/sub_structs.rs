use ethers::types::H160;
use serde::{Deserialize, Deserializer};

// Custom deserializer for liquidation_px that converts "NaN" to None
fn deserialize_liquidation_px<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.and_then(|s| if s == "NaN" { None } else { Some(s) }))
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Leverage {
    #[serde(rename = "type")]
    pub type_string: String,
    pub value: u32,
    pub raw_usd: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CumulativeFunding {
    pub all_time: String,
    pub since_open: String,
    pub since_change: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PositionData {
    pub coin: String,
    pub entry_px: Option<String>,
    pub leverage: Leverage,
    #[serde(deserialize_with = "deserialize_liquidation_px")]
    pub liquidation_px: Option<String>,
    pub margin_used: String,
    pub position_value: String,
    pub return_on_equity: String,
    pub szi: String,
    pub unrealized_pnl: String,
    pub max_leverage: u32,
    pub cum_funding: CumulativeFunding,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AssetPosition {
    pub position: PositionData,
    #[serde(rename = "type")]
    pub type_string: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    pub n: u64,
    pub px: String,
    pub sz: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Delta {
    #[serde(rename = "type")]
    pub type_string: String,
    pub coin: String,
    pub usdc: String,
    pub szi: String,
    pub funding_rate: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DailyUserVlm {
    pub date: String,
    pub exchange: String,
    pub user_add: String,
    pub user_cross: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FeeSchedule {
    pub add: String,
    pub cross: String,
    pub referral_discount: String,
    pub tiers: Tiers,
}

#[derive(Deserialize, Debug)]
pub struct Tiers {
    pub mm: Vec<Mm>,
    pub vip: Vec<Vip>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mm {
    pub add: String,
    pub maker_fraction_cutoff: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vip {
    pub add: String,
    pub cross: String,
    pub ntl_cutoff: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserTokenBalance {
    pub coin: String,
    pub hold: String,
    pub total: String,
    pub entry_ntl: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderInfo {
    pub order: BasicOrderInfo,
    pub status: String,
    pub status_timestamp: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BasicOrderInfo {
    pub coin: String,
    pub side: String,
    pub limit_px: String,
    pub sz: String,
    pub oid: u64,
    pub timestamp: u64,
    pub trigger_condition: String,
    pub is_trigger: bool,
    pub trigger_px: String,
    pub is_position_tpsl: bool,
    pub reduce_only: bool,
    pub order_type: String,
    pub orig_sz: String,
    pub tif: String,
    pub cloid: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Referrer {
    pub referrer: H160,
    pub code: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReferrerState {
    pub stage: String,
    pub data: ReferrerData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReferrerData {
    pub required: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_position_data_liquidation_px_deserialization() {
        // Test case with "NaN" liquidation price
        let json_with_nan = json!({
            "coin": "BTC",
            "entryPx": "50000",
            "leverage": {
                "type": "cross",
                "value": 10,
                "rawUsd": "500000"
            },
            "liquidationPx": "NaN",
            "marginUsed": "5000",
            "positionValue": "50000",
            "returnOnEquity": "0.1",
            "szi": "1",
            "unrealizedPnl": "1000",
            "maxLeverage": 20,
            "cumFunding": {
                "allTime": "100",
                "sinceOpen": "10",
                "sinceChange": "5"
            }
        });

        let position: PositionData = serde_json::from_value(json_with_nan).unwrap();
        assert!(
            position.liquidation_px.is_none(),
            "Expected None for NaN liquidation price"
        );

        // Test case with valid liquidation price
        let json_with_price = json!({
            "coin": "BTC",
            "entryPx": "50000",
            "leverage": {
                "type": "cross",
                "value": 10,
                "rawUsd": "500000"
            },
            "liquidationPx": "45000",
            "marginUsed": "5000",
            "positionValue": "50000",
            "returnOnEquity": "0.1",
            "szi": "1",
            "unrealizedPnl": "1000",
            "maxLeverage": 20,
            "cumFunding": {
                "allTime": "100",
                "sinceOpen": "10",
                "sinceChange": "5"
            }
        });

        let position: PositionData = serde_json::from_value(json_with_price).unwrap();
        assert_eq!(
            position.liquidation_px,
            Some("45000".to_string()),
            "Expected Some(45000) for valid liquidation price"
        );
    }
}
