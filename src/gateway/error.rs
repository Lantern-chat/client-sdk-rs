use tokio_tungstenite::tungstenite::Error as WSError;

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("WS Error: {0}")]
    WSError(#[from] WSError),

    #[error("Gateway Disconnected")]
    Disconnected,

    #[error("Json Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[cfg(feature = "msgpack")]
    #[error("MsgPack Encode Error: {0}")]
    MsgPackEncodeError(#[from] rmp_serde::encode::Error),

    #[cfg(feature = "msgpack")]
    #[error("MsgPack Decode Error: {0}")]
    MsgPackDecodeError(#[from] rmp_serde::decode::Error),

    #[error("Compression Error")]
    CompressionError,

    #[error("Close Error: {0:?}")]
    CloseError(GatewayErrorCode),
}

impl GatewayError {
    pub(crate) fn should_reconnect(&self) -> bool {
        use GatewayErrorCode as C;

        matches!(
            self,
            Self::Disconnected
                | Self::WSError(WSError::AlreadyClosed | WSError::ConnectionClosed)
                | Self::CloseError(C::UnknownError | C::DecodeError)
        )
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[derive(enum_primitive_derive::Primitive)]
#[repr(u16)]
pub enum GatewayErrorCode {
    UnknownError        = 4000,
    UnknownOpcode       = 4001,
    DecodeError         = 4002,
    NotAuthenticated    = 4003,
    AuthFailed          = 4004,
}
