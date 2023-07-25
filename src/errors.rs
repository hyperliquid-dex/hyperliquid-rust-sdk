use std::error::Error;
use std::fmt::{Display, Formatter, Result};

use reqwest::header::HeaderMap;
#[derive(Debug, Clone)]
pub(crate) struct ClientError {
    pub(crate) status_code: u16,
    pub(crate) error_code: Option<u16>,
    pub(crate) error_message: String,
    pub(crate) headers: HeaderMap,
    pub(crate) error_data: Option<String>,
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let Self {
            status_code,
            error_code,
            error_message,
            headers,
            error_data,
        } = self;
        write!(f, "Client error: status code: {status_code}, error code: {error_code:?}, error message: {error_message}, headers: {headers:?}, error data: {error_data:?}")
    }
}

impl Error for ClientError {}

#[derive(Debug, Clone)]
pub(crate) struct ServerError {
    pub(crate) status_code: u16,
    pub(crate) error_message: String,
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Server error: status code: {}, error message: {}",
            self.status_code, self.error_message
        )
    }
}

impl Error for ServerError {}

#[derive(Debug, Clone)]
pub(crate) struct ChainError;

impl Display for ChainError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Chain type not allowed for this function")
    }
}

impl Error for ChainError {}

#[derive(Debug, Clone)]
pub(crate) struct AssetNotFoundError;

impl Display for AssetNotFoundError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Asset not found!")
    }
}

impl Error for AssetNotFoundError {}
