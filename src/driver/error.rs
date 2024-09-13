use crate::api::error::{ApiError, ApiErrorCode};

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("Reqwest Error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Format Error")]
    FormatError(#[from] core::fmt::Error),

    #[error("Url Parse Error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Url Encoding Error: {0}")]
    UrlEncodingError(#[from] serde_urlencoded::ser::Error),

    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[cfg(feature = "cbor")]
    #[error("CBOR Encode Error: {0}")]
    CborEncodeError(#[from] ciborium::ser::Error<std::io::Error>),
    #[cfg(feature = "cbor")]
    #[error("CBOR Encode Error: {0}")]
    CborDecodeError(#[from] ciborium::de::Error<std::io::Error>),

    #[error("Api Error: {0:?}")]
    ApiError(ApiError),

    #[error("Generic Driver Error: {0}")]
    GenericDriverError(http::StatusCode),

    #[error("Missing Authorization")]
    MissingAuthorization,

    #[error("Invalid Header Value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    #[error("Parse Int Error: {0}")]
    ParseIntError(#[from] core::num::ParseIntError),

    #[error("Header Parse Error: {0}")]
    HeaderParseError(#[from] http::header::ToStrError),
}

impl DriverError {
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        match self {
            DriverError::ApiError(err) => err.code == ApiErrorCode::NotFound,
            DriverError::ReqwestError(err) => err.status() == Some(reqwest::StatusCode::NOT_FOUND),
            _ => false,
        }
    }
}
