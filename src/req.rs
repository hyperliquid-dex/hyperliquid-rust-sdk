use crate::{prelude::*, Error};
use reqwest::{Client, Response};
use serde::Deserialize;

#[derive(Deserialize)]
struct ErrorData {
    data: String,
    code: u16,
    msg: String,
}

pub struct HttpClient {
    pub client: Client,
    pub base_url: String,
}

async fn parse_response(response: Response) -> Result<String> {
    let status_code = response.status().as_u16();
    let headers = response.headers().clone();
    let text = response
        .text()
        .await
        .map_err(|e| Error::GenericRequest(e.to_string()))?;

    if status_code < 400 {
        return Ok(text);
    }
    let error_data = serde_json::from_str::<ErrorData>(&text);

    if status_code >= 400 && status_code < 500 {
        let client_error = match error_data {
            Ok(error_data) => Error::ClientRequest {
                status_code,
                error_code: Some(error_data.code),
                error_message: error_data.msg,
                headers,
                error_data: Some(error_data.data),
            },
            Err(err) => Error::ClientRequest {
                status_code,
                error_message: text,
                headers,
                error_code: None,
                error_data: Some(err.to_string()),
            },
        };
        return Err(client_error);
    }

    Err(Error::ServerRequest {
        status_code,
        error_message: text,
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
}
