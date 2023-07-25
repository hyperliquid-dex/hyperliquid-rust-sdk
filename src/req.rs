use crate::errors::{ClientError, ServerError};
use reqwest::{Client, Response};
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
struct ErrorData {
    data: String,
    code: u16,
    msg: String,
}

pub struct HttpClient<'a> {
    pub client: Client,
    pub base_url: &'a str,
}

async fn parse_response(response: Response) -> Result<String, Box<dyn Error>> {
    let status_code = response.status().as_u16();
    let headers = response.headers().clone();
    let text = response.text().await?;

    if status_code < 400 {
        return Ok(text);
    }
    let error_data = serde_json::from_str::<ErrorData>(&text);

    if status_code >= 400 && status_code < 500 {
        let client_error = match error_data {
            Ok(error_data) => ClientError {
                status_code,
                error_code: Some(error_data.code),
                error_message: error_data.msg,
                headers,
                error_data: Some(error_data.data),
            },
            Err(err) => ClientError {
                status_code,
                error_message: text,
                headers,
                error_code: None,
                error_data: Some(err.to_string()),
            },
        };
        return Err(Box::new(client_error));
    }

    Err(Box::new(ServerError {
        status_code,
        error_message: text,
    }))
}

impl<'a> HttpClient<'a> {
    pub async fn post(
        &self,
        url_path: &'static str,
        data: String,
    ) -> Result<String, Box<dyn Error>> {
        let full_url = format!("{}{url_path}", self.base_url);
        let request = self
            .client
            .post(full_url)
            .header("Content-Type", "application/json")
            .body(data)
            .build()
            .unwrap();
        let result = self.client.execute(request).await.unwrap();
        parse_response(result).await
    }
}
