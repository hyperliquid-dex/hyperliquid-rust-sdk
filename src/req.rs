use crate::{prelude::*, BaseUrl, Error};
use reqwest::{Client, Response};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ErrorData {
    data: String,
    code: u16,
    msg: String,
}

#[derive(Debug)]
pub struct HttpClient {
    pub client: Client,
    pub base_url: String,
}

async fn parse_response(response: Response) -> Result<String> {
    let status_code = response.status().as_u16();
    let text = response
        .text()
        .await
        .map_err(|e| Error::GenericRequest(e.to_string()))?;

    if status_code < 400 {
        return Ok(text);
    }
    if let Ok(err) = serde_json::from_str::<ErrorData>(&text) {
        let client_error = Error::ClientRequest {
            message: err.msg,
            error_code: Some(err.code as i64),
            error_data: Some(err.data),
        };
        return Err(client_error);
    }

    Err(Error::ServerRequest {
        message: format!("Server error (status: {}): {}", status_code, text),
    })
}

impl HttpClient {
    pub async fn post(&self, url_path: &'static str, data: String) -> Result<String> {
        let full_url = format!("{}{url_path}", self.base_url);
        let request = self
            .client
            .post(full_url)
            .header("Content-Type", "application/json")
            .body(data)
            .build()
            .map_err(|e| Error::GenericRequest(e.to_string()))?;
        let result = self
            .client
            .execute(request)
            .await
            .map_err(|e| Error::GenericRequest(e.to_string()))?;
        parse_response(result).await
    }

    pub fn is_mainnet(&self) -> bool {
        self.base_url == BaseUrl::Mainnet.get_url()
    }
}
