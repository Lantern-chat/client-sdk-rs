use tokio_tungstenite::tungstenite::Error as WSError;

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("WS Error: {0}")]
    WSError(#[from] WSError),

    #[error("Gateway Disconnected")]
    Disconnected,
}

impl GatewayError {
    pub(crate) fn is_close(&self) -> bool {
        matches!(
            self,
            Self::Disconnected | Self::WSError(WSError::AlreadyClosed | WSError::ConnectionClosed)
        )
    }
}
