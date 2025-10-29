use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponse {
    pub channel: String,
    pub data: PostResponseData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponseData {
    pub id: u64,
    pub response: PostResponseDataResponse,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponseDataResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub payload: PostResponsePayload,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponsePayload {
    pub status: String,
    pub response: PostResponsePayloadResponse,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponsePayloadResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub data: Option<PostResponsePayloadData>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponsePayloadData {
    pub statuses: Vec<PostResponseStatus>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PostResponseStatus {
    Simple(String),
    Detailed {
        error: Option<String>,
        filled: Option<FilledStatus>,
        resting: Option<RestingStatus>,
    },
}

impl PostResponseStatus {
    // Returns the oid of the order either from the filled or resting status
    // If neither is present, returns `None`
    pub fn get_oid(&self) -> Option<String> {
        match self {
            PostResponseStatus::Simple(_) => None,
            PostResponseStatus::Detailed { filled, resting, .. } => {
                filled
                    .as_ref()
                    .map(|filled| filled.oid.to_string())
                    .or_else(|| resting.as_ref().map(|resting| resting.oid.to_string()))
            }
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilledStatus {
    pub total_sz: String,
    pub avg_px: String,
    pub oid: u64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestingStatus {
    pub oid: u64,
    pub cloid: Option<String>,
}
