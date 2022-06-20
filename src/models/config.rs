use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ServerConfig {
    pub hcaptcha_sitekey: String,

    /// CDN Domain
    pub cdn: String,

    /// Minimum user age (in years)
    pub min_age: u8,

    /// If the serve should require HTTPS
    pub secure: bool,

    pub limits: ServerLimits,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ServerLimits {
    pub max_upload_size: u64,
    pub max_avatar_size: u32,
    pub max_avatar_pixels: u32,
}
