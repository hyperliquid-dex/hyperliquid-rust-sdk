use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Universe {
    pub universe: Vec<UniverseItem>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UniverseItem {
    pub name: String,
    sz_decimals: u32,
    max_leverage: u32,
    only_isolated: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerpetualAssetContext {
    day_ntl_vlm: String,
    funding: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    impact_pxs: Option<Vec<String>>,
    mark_px: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mid_px: Option<String>,
    open_interest: String,
    oracle_px: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    premium: Option<String>,
    prev_day_px: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Leverage {
    #[serde(rename = "type")]
    pub type_string: String,
    pub value: u32,
    pub raw_usd: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PositionData {
    pub coin: String,
    pub entry_px: Option<String>,
    pub leverage: Leverage,
    pub liquidation_px: Option<String>,
    pub margin_used: String,
    pub position_value: String,
    pub return_on_equity: String,
    pub szi: String,
    pub unrealized_pnl: String,
}

#[derive(Deserialize, Debug)]
pub struct AssetPosition {
    pub position: PositionData,
    #[serde(rename = "type")]
    pub type_string: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarginSummary {
    pub account_value: String,
    pub total_margin_used: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    pub n: u64,
    pub px: String,
    pub sz: String,
}
