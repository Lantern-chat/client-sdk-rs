use super::*;

command! {
    /// Gets the global server configuration
    -struct GetServerConfig -> ServerConfig: GET("config") {}
}
