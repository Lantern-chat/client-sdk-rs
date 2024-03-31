pub use crate::driver::Encoding;

const fn default_compress() -> bool {
    true
}

/// Query parameters for the gateway, used to configure the connection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GatewayQueryParams {
    /// Encoding method for each individual websocket message
    #[serde(alias = "e")]
    pub encoding: Encoding,

    /// Whether to compress individual messages
    #[serde(alias = "c")]
    pub compress: bool,
}

impl Default for GatewayQueryParams {
    fn default() -> Self {
        GatewayQueryParams {
            encoding: Encoding::default(),
            compress: default_compress(),
        }
    }
}
