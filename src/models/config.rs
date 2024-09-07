use super::*;

pub type HCaptchaSiteKey = FixedStr<36>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    archive(check_bytes)
)]
pub struct ServerConfig {
    pub hcaptcha_sitekey: HCaptchaSiteKey,

    /// CDN Domain
    pub cdn: SmolStr,

    /// Minimum user age (in years)
    pub min_age: u8,

    /// If the serve should require HTTPS
    pub secure: bool,

    pub limits: ServerLimits,

    /// If true, use a "camo"/camouflage route provided at "{cdn}/camo/base64_url/url_signature"
    pub camo: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    archive(check_bytes)
)]
pub struct ServerLimits {
    pub max_upload_size: u64,
    pub max_avatar_size: u32,
    pub max_banner_size: u32,
    pub max_avatar_pixels: u32,
    pub max_banner_pixels: u32,
    pub avatar_width: u32,
    pub banner_width: u32,
    pub banner_height: u32,
}
