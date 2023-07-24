use reqwest;
use serde_json;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ErrorData {
    data: String,
    code: u16,
    msg: String,
}

pub async fn post(client: &reqwest::Client, full_url: &String, data: String) -> String {
    let request = client
        .post(full_url)
        .header("Content-Type", "application/json")
        .body(data)
        .build()
        .unwrap();
    let result = client.execute(request).await.unwrap();
    process_response(result).await
}

async fn process_response(response: reqwest::Response) -> String {
    let status_code = response.status().as_u16();
    let headers = response.headers().clone();
    let text = response.text().await.unwrap();

    if status_code < 400 {
        return text;
    }
    let error_data = serde_json::from_str::<ErrorData>(&text);

    if (400..500).contains(&status_code) {
        match error_data {
            Ok(error_data) => panic!("Client error: status code: {}, error code: {}, error message: {}, response headers: {:?}, error data: {}", status_code, error_data.code, error_data.msg, headers, error_data.data), 
            Err(_) => panic!("Client error: status code: {}, error message: {}, response headers: {:?}", status_code, text, headers)
        }
    }
    panic!(
        "Server error: status code: {}, response text: {}",
        status_code, text
    );
}
