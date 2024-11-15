use super::*;

command! { Config;

    /// Gets the global server configuration
    -struct GetServerConfig -> One ServerConfig: GET("config") {}
}
