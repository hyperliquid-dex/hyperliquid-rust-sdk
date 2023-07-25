use crate::info::open_order::OpenOrdersResponse;
use crate::info::user_state::UserStateResponse;
use crate::{consts::MAINNET_API_URL, req::HttpClient};
use reqwest::Client;
use serde::Serialize;
use std::error::Error;

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum InfoRequest<'a> {
    #[serde(rename = "clearinghouseState")]
    UserState {
        user: &'a str,
    },
    OpenOrders {
        user: &'a str,
    },
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

    pub async fn open_orders(
        &self,
        address: &str,
    ) -> Result<Vec<OpenOrdersResponse>, Box<dyn Error>> {
        let input = InfoRequest::OpenOrders { user: address };
        let data = serde_json::to_string(&input)?;

        let return_data = self.http_client.post("/info", data).await?;
        Ok(serde_json::from_str(&return_data)?)
    }

    pub async fn user_state(&self, address: &str) -> Result<UserStateResponse, Box<dyn Error>> {
        let input = InfoRequest::UserState { user: address };
        let data = serde_json::to_string(&input)?;

        let return_data = self.http_client.post("/info", data).await?;
        Ok(serde_json::from_str(&return_data)?)
    }
}
