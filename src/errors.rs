use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client request error: {message} (code: {error_code:?}, data: {error_data:?})")]
    ClientRequest {
        message: String,
        error_code: Option<i64>,
        error_data: Option<String>,
    },

    #[error("Server request error: {message}")]
    ServerRequest { message: String },

    #[error("Chain not allowed")]
    ChainNotAllowed,

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Alloy conversion error: {0}")]
    AlloyConversion(String),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("EIP712 error: {0}")]
    Eip712(String),

    #[error("Generic parse error: {0}")]
    GenericParse(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Vault address not found: {0}")]
    VaultAddressNotFound(String),

    #[error("Float string parse error: {0}")]
    FloatStringParse(String),

    #[error("Generic request error: {0}")]
    GenericRequest(String),

    #[error("Subscription not found")]
    SubscriptionNotFound,

    #[error("WS manager not instantiated")]
    WsManagerNotFound,

    #[error("WS send error: {0:?}")]
    WsSend(String),

    #[error("Reader data not found")]
    ReaderDataNotFound,

    #[error("Reader error: {0:?}")]
    GenericReader(String),

    #[error("Reader text conversion error: {0:?}")]
    ReaderTextConversion(String),

    #[error("Order type not found")]
    OrderTypeNotFound,

    #[error("Issue with generating random data: {0:?}")]
    RandGen(String),

    #[error("Private key parse error: {0}")]
    PrivateKeyParse(String),

    #[error("Cannot subscribe to multiple user events")]
    UserEvents,

    #[error("RMP parse error: {0}")]
    RmpParse(String),

    #[error("JSON parse error: {0}")]
    JsonParse(String),

    #[error("Websocket error: {0}")]
    Websocket(String),

    #[error("Signature failure: {0}")]
    SignatureFailure(String),

    #[error("Alloy signer error: {0}")]
    AlloySignerError(String),
}

impl From<alloy::signers::Error> for Error {
    fn from(err: alloy::signers::Error) -> Self {
        Error::AlloySignerError(err.to_string())
    }
}

impl From<fn(String) -> Error> for Error {
    fn from(_: fn(String) -> Error) -> Self {
        Error::AssetNotFound("Asset not found".to_string())
    }
}
