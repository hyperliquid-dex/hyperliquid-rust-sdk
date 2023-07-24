use std::error::Error;
use std::fmt::{Display, Formatter, Result};

use reqwest::header::HeaderMap;
#[derive(Debug, Clone)]
pub struct ClientError {
    pub status_code: u16,
    pub error_code: Option<u16>,
    pub error_message: String,
    pub headers: HeaderMap,
    pub error_data: Option<String>,
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Client error: status code: {}, error code: {:?}, error message: {}, headers: {:?}, error data: {:?}", self.status_code, self.error_code, self.error_message, self.headers, self.error_data)
    }
}

impl Error for ClientError {}

#[derive(Debug, Clone)]
pub struct ServerError {
    pub status_code: u16,
    pub error_message: String,
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
