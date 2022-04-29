use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ServerConfig {
    pub hcaptcha_sitekey: String,

    /// CDN Domain
    pub cdn: String,

    /// Minimum user age (in years)
    pub min_age: u8,
}
