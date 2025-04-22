use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Default, Deserialize, Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuilderInfo {
    #[serde(rename = "b")]
    pub builder: String,
    #[serde(rename = "f")]
    pub fee: u64,
}
