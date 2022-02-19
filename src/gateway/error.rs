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
}

impl GatewayError {
    pub(crate) fn is_close(&self) -> bool {
        matches!(
            self,
            Self::Disconnected | Self::WSError(WSError::AlreadyClosed | WSError::ConnectionClosed)
        )
    }
}
