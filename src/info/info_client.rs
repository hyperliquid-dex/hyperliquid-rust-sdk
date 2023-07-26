use crate::info::open_order::OpenOrdersResponse;
use crate::info::user_state::UserStateResponse;
use crate::{consts::MAINNET_API_URL, meta::Meta, prelude::*, req::HttpClient, Error};
use ethers::types::H160;
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum InfoRequest {
    #[serde(rename = "clearinghouseState")]
    UserState {
        user: H160,
    },
    OpenOrders {
        user: H160,
    },
    Meta,
}

pub struct InfoClient<'a> {
    pub http_client: HttpClient<'a>,
}

impl<'a> InfoClient<'a> {
    pub fn new(client: Option<Client>, base_url: Option<&'a str>) -> Self {
        let client = client.unwrap_or_else(Client::new);
        let base_url = base_url.unwrap_or(MAINNET_API_URL);

        InfoClient {
            http_client: HttpClient { client, base_url },
        }
    }

    pub async fn open_orders(&self, address: H160) -> Result<Vec<OpenOrdersResponse>> {
        let input = InfoRequest::OpenOrders { user: address };
        let data = serde_json::to_string(&input).map_err(|e| Error::JsonParse(e.to_string()))?;

        let return_data = self.http_client.post("/info", data).await?;
        serde_json::from_str(&return_data).map_err(|e| Error::JsonParse(e.to_string()))
    }

    pub async fn user_state(&self, address: H160) -> Result<UserStateResponse> {
        let input = InfoRequest::UserState { user: address };
        let data = serde_json::to_string(&input).map_err(|e| Error::JsonParse(e.to_string()))?;

        let return_data = self.http_client.post("/info", data).await?;
        serde_json::from_str(&return_data).map_err(|e| Error::JsonParse(e.to_string()))
    }

    pub async fn meta(&self) -> Result<Meta> {
        let input = InfoRequest::Meta;
        let data = serde_json::to_string(&input).map_err(|e| Error::JsonParse(e.to_string()))?;

        let return_data = self.http_client.post("/info", data).await?;
        serde_json::from_str(&return_data).map_err(|e| Error::JsonParse(e.to_string()))
    }
}
