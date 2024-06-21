use ethers::abi::ethereum_types::H128;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Meta {
    pub universe: Vec<AssetMeta>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SpotMeta {
    pub universe: Vec<SpotAssetMeta>,
    pub tokens: Vec<TokenInfo>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SpotMetaAndAssetCtxs {
    SpotMeta(SpotMeta),
    Context(Vec<SpotAssetContext>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotAssetContext {
    pub day_ntl_vlm: String,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub prev_day_px: String,
    pub circulating_supply: String,
    pub coin: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetMeta {
    pub name: String,
    pub sz_decimals: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotAssetMeta {
    pub tokens: [usize; 2],
    pub name: String,
    pub index: usize,
    pub is_canonical: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub name: String,
    pub sz_decimals: u8,
    pub wei_decimals: u8,
    pub index: usize,
    pub token_id: H128,
    pub is_canonical: bool,
}
