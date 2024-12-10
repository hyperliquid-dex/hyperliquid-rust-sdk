use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuilderInfo {
    #[serde(rename = "b")]
    pub builder: String,
    #[serde(rename = "f")]
    pub fee: f64,
}
