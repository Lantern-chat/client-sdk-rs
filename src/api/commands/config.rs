use super::*;

command! {
    /// Gets the global server configuration
    -struct GetServerConfig -> One ServerConfig: GET("config") {}
}
