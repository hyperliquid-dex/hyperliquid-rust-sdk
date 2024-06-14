use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Meta {
    pub universe: Vec<AssetMeta>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SpotMeta {
    pub universe: Vec<SpotAssetMeta>,
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
    pub name: String,
}
