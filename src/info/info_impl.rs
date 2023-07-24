use crate::info::open_order::OpenOrdersResponse;
use crate::info::user_state::UserStateResponse;
use crate::{consts::MAINNET_API_URL, req::ClientAndBaseUrl};
use serde::Serialize;
use std::error::Error;
use reqwest::Client; 

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum InfoRequest {
    #[serde(rename = "clearinghouseState")]
    UserState {
        user: String,
    },
    OpenOrders {
        user: String,
    },
}

pub struct Info {
    pub client_and_base_url: ClientAndBaseUrl,
}

impl Info {
    pub fn new(client: Option<Client>, base_url: Option<String>) -> Self {
        let client = client.unwrap_or_else(Client::new);
        let base_url = base_url.unwrap_or_else(|| MAINNET_API_URL.to_owned());

        Info {
            client_and_base_url: ClientAndBaseUrl { client, base_url },
        }
    }

    pub async fn open_orders(
        &self,
        address: String,
    ) -> Result<Vec<OpenOrdersResponse>, Box<dyn Error>> {
        let input = InfoRequest::OpenOrders { user: address };
        let data = serde_json::to_string(&input)?;

        let return_data = self
            .client_and_base_url
            .post("/info".to_string(), data)
            .await?;
        Ok(serde_json::from_str::<Vec<OpenOrdersResponse>>(&return_data)?)
    }

    pub async fn user_state(&self, address: String) -> Result<UserStateResponse, Box<dyn Error>> {
        let input = InfoRequest::UserState { user: address };
        let data = serde_json::to_string(&input)?;

        let return_data = self
            .client_and_base_url
            .post("/info".to_string(), data)
            .await?;
        Ok(serde_json::from_str::<UserStateResponse>(&return_data)?)
    }
}
