use tokio_tungstenite::tungstenite::Error as WSError;

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("WS Error: {0}")]
    WSError(#[from] WSError),

    #[error("Gateway Disconnected")]
    Disconnected,

    #[error("Exceeded Reconnect Limit of {0} Attempts")]
    ReconnectLimitExceeded(usize),

    #[error("Json Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[cfg(feature = "cbor")]
    #[error("CBOR Encode Error: {0}")]
    CborEncodeError(#[from] ciborium::ser::Error<std::io::Error>),
    #[cfg(feature = "cbor")]
    #[error("CBOR Encode Error: {0}")]
    CborDecodeError(#[from] ciborium::de::Error<std::io::Error>),

    #[error("Compression Error")]
    CompressionError,

    #[error("Close Error: {0:?}")]
    CloseError(GatewayErrorCode),
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema_repr))]
#[derive(enum_primitive_derive::Primitive)]
#[repr(u16)]
pub enum GatewayErrorCode {
    UnknownError        = 4000,
    UnknownOpcode       = 4001,
    DecodeError         = 4002,
    NotAuthenticated    = 4003,
    AuthFailed          = 4004,
}
