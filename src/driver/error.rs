#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("Reqwest Error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Format Error")]
    FormatError(#[from] std::fmt::Error),

    #[error("Url Parse Error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Url Encoding Error: {0}")]
    UrlEncodingError(#[from] serde_urlencoded::ser::Error),

    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[cfg(feature = "msgpack")]
    #[error("MsgPack Encode Error: {0}")]
    MsgPackEncodeError(#[from] rmp_serde::encode::Error),

    #[cfg(feature = "msgpack")]
    #[error("MsgPack Decode Error: {0}")]
    MsgPackDecodeError(#[from] rmp_serde::decode::Error),

    #[error("Api Error: {0:?}")]
    ApiError(crate::api::error::ApiError),

    #[error("Generic Driver Error: {0}")]
    GenericDriverError(http::StatusCode),

    #[error("Missing Authorization")]
    MissingAuthorization,

    #[error("Invalid Header Value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    #[error("Parse Int Error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Header Parse Error: {0}")]
    HeaderParseError(#[from] http::header::ToStrError),
}
