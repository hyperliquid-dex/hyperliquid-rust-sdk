use crate::{
    consts::MAINNET_API_URL,
    info::{open_order::OpenOrdersResponse, user_state::UserStateResponse},
    meta::Meta,
    prelude::*,
    req::HttpClient,
    ws::{SubscriptionType, WsManager},
    Error,
};

use ethers::types::H160;
use reqwest::Client;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;

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
    pub(crate) http_client: HttpClient<'a>,
    pub(crate) ws_manager: Option<WsManager>,
}

impl<'a> InfoClient<'a> {
    pub async fn new(
        client: Option<Client>,
        base_url: Option<&'a str>,
        skip_websocket: bool,
    ) -> Result<InfoClient> {
        let client = client.unwrap_or_else(Client::new);
        let base_url = base_url.unwrap_or(MAINNET_API_URL);

        let ws_manager = if !skip_websocket {
            Some(WsManager::new(format!("ws{}/ws", &base_url[4..])).await?)
        } else {
            None
        };

        Ok(InfoClient {
            http_client: HttpClient { client, base_url },
            ws_manager,
        })
    }

    pub async fn subscribe(
        &mut self,
        subscription: SubscriptionType,
        sender_channel: UnboundedSender<String>,
    ) -> Result<u32> {
        let identifier =
            serde_json::to_string(&subscription).map_err(|e| Error::JsonParse(e.to_string()))?;

        self.ws_manager
            .as_mut()
            .ok_or(Error::WsManagerNotFound)?
            .add_subscription(identifier, sender_channel)
            .await
    }

    pub async fn unsubscribe(&mut self, subscription_id: u32) -> Result<()> {
        self.ws_manager
            .as_mut()
            .ok_or(Error::WsManagerNotFound)?
            .remove_subscription(subscription_id)
            .await
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
