use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct UsdcTransfer {
    pub(crate) chain: String,
    pub(crate) payload: serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateLeverage {
    pub(crate) asset: u32,
    pub(crate) is_cross: bool,
    pub(crate) leverage: u32,
}
