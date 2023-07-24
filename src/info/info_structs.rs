use serde;

#[derive(serde::Serialize)]
pub struct OpenOrderInput {
    #[serde(rename = "type")]
    pub request_type: String,
    pub user: String, 
}

#[derive(serde::Deserialize, Debug)]
pub struct Order {
    pub coin: String, 
    #[serde(rename = "limitPx")]
    pub limit_px: String, 
    pub oid: i32, 
    pub side: String, 
    pub sz: String, 
    pub timestamp: u64, 
}