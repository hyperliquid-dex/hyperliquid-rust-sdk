use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::TryFrom;

/// Generic WebSocket response that can be either a success or error response
#[derive(Debug, Clone)]
pub(crate) enum WsResponse {
    Post(WsPostResponse),
    Error(WsErrorResponse),
    Other(serde_json::Value),
}

impl TryFrom<Value> for WsResponse {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, <Self as std::convert::TryFrom<Value>>::Error> {
        if let Ok(post_response) = serde_json::from_value::<WsPostResponse>(value.clone()) {
            return Ok(WsResponse::Post(post_response));
        } else if let Ok(error_response) = serde_json::from_value::<WsErrorResponse>(value.clone())
        {
            return Ok(WsResponse::Error(error_response));
        } else {
            return Ok(WsResponse::Other(value));
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct WsErrorResponse {
    pub channel: String,
    pub data: WsErrorResponseData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct WsErrorResponseData {
    pub id: u64,
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct WsRequest {
    #[serde(rename = "type")]
    pub type_: String,
    pub payload: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct WsPostRequest {
    pub method: String,
    pub id: u64,
    pub request: WsRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WsPostResponse {
    pub channel: String,
    pub data: WsPostResponseData,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct WsPostResponseData {
    pub id: u64,
    pub response: serde_json::Value,
}
