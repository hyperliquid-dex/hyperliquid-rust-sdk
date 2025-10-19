use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

use crate::{prelude::*, BaseUrl, Error};

#[derive(Deserialize, Debug)]
struct ErrorData {
    data: String,
    code: u16,
    msg: String,
}

#[derive(Debug)]
pub struct HttpClient {
    pub client: Client,
    pub base_url_enum: BaseUrl,
    pub base_url: String,
    pub ltp_api_key: Option<String>,
    pub ltp_api_secret: Option<String>,
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
    let error_data = serde_json::from_str::<ErrorData>(&text);
    if (400..500).contains(&status_code) {
        let client_error = match error_data {
            Ok(error_data) => Error::ClientRequest {
                status_code,
                error_code: Some(error_data.code),
                error_message: error_data.msg,
                error_data: Some(error_data.data),
            },
            Err(err) => Error::ClientRequest {
                status_code,
                error_message: text,
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
    pub fn new(
        client: Client,
        base_url_enum: BaseUrl,
        base_url: String,
        ltp_api_key: Option<String>,
        ltp_api_secret: Option<String>,
    ) -> Self {
        Self {
            client,
            base_url_enum,
            base_url,
            ltp_api_key,
            ltp_api_secret,
        }
    }

    pub async fn post(&self, url_path: &'static str, data: String) -> Result<String> {
        let full_url = format!("{}{url_path}", self.base_url);
        println!("full_url: {}", full_url);
        let mut request_builder = self.client.post(full_url);
        
        if self.base_url_enum == BaseUrl::LTP {
            // LTP-specific authentication logic
            if let (Some(api_key), Some(api_secret)) = (&self.ltp_api_key, &self.ltp_api_secret) {
                // Build request body for LTP in format {"body": "json_dumps(data)"}
                let new_body = if !data.is_empty() {
                    // Escape the JSON string by replacing " with \"
                    let escaped_data = data.replace("\"", "\\\"");
                    format!("{{\"body\":\"{}\"}}", escaped_data)
                } else {
                    "{}".to_string()
                };
                
                // Build encryption string
                let mut to_encrypt = String::new();
                if !new_body.is_empty() && new_body != "{}" {
                    // Since body only has one key "body", we can directly format it
                    to_encrypt.push_str(&format!("body={}&", data));
                }
                
                // Add timestamp
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                to_encrypt.push_str(&now.to_string());
                
                // Create HMAC signature
                let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())
                    .map_err(|e| Error::GenericRequest(format!("HMAC key error: {}", e)))?;
                mac.update(to_encrypt.as_bytes());
                let signature = hex::encode(mac.finalize().into_bytes());

                println!("new_body: {}", new_body);
                println!("to_encrypt: {}", to_encrypt);
                
                // Print headers in the expected format
                println!("Headers: {{'Content-Type': 'application/json', 'X-MBX-APIKEY': '{}', 'signature': '{}', 'nonce': '{}'}}", 
                    api_key, signature, now);
                
                // Set request headers for LTP
                request_builder = request_builder
                    .header("Content-Type", "application/json")
                    .header("X-MBX-APIKEY", api_key)
                    .header("signature", signature)
                    .header("nonce", now.to_string())
                    .body(new_body);
            } else {
                return Err(Error::GenericRequest("LTP API key and secret are required for LTP base URL".to_string()));
            }
        } else {
            // Standard request for non-LTP URLs
            request_builder = request_builder
                .header("Content-Type", "application/json")
                .body(data);
        }
        
        let request = request_builder
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
        self.base_url == BaseUrl::Mainnet.get_url() || self.base_url == BaseUrl::LTP.get_url()
    }
}
