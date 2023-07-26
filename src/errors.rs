use reqwest::header::HeaderMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client error: status code: {status_code}, error code: {error_code:?}, error message: {error_message}, headers: {headers:?}, error data: {error_data:?}")]
    ClientRequest {
        status_code: u16,
        error_code: Option<u16>,
        error_message: String,
        headers: HeaderMap,
        error_data: Option<String>,
    },
    #[error("Server error: status code: {status_code}, error message: {error_message}")]
    ServerRequest {
        status_code: u16,
        error_message: String,
    },
    #[error("Generic request error: {0:?}")]
    GenericRequest(String),
    #[error("Chain type not allowed for this function")]
    ChainNotAllowed,
    #[error("Asset not found!")]
    AssetNotFound,
    #[error("Error from Eip712 struct: {0:?}")]
    Eip712(String),
    #[error("Json parse error: {0:?}")]
    JsonParse(String),
    #[error("Generic parse error: {0:?}")]
    GenericParse(String),
    #[error("Wallet error: {0:?}")]
    Wallet(String),
}
