use crate::driver::DriverError;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Driver Error: {0}")]
    DriverError(DriverError),

    #[error("Invalid Bearer Token")]
    InvalidBearerToken,

    #[error("Api Error: {0:?}")]
    ApiError(crate::api::error::ApiError),

    #[error("File Too Large")]
    FileTooLarge,

    #[error("Not a file")]
    NotAFile,
}

impl From<DriverError> for ClientError {
    fn from(err: DriverError) -> ClientError {
        match err {
            DriverError::ApiError(err) => ClientError::ApiError(err),
            _ => ClientError::DriverError(err),
        }
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> ClientError {
        ClientError::DriverError(DriverError::ReqwestError(err))
    }
}
