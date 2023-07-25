#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrdersResponse {
    pub coin: String,
    pub limit_px: String,
    pub oid: i32,
    pub side: String,
    pub sz: String,
    pub timestamp: u64,
}
