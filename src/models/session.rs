use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Session {
    /// Auth token encoded as base-64
    pub auth: AuthToken,
    /// Expiration timestamp encoded with RFC 3339
    pub expires: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AnonymousSession {
    /// Expiration timestamp encoded with RFC 3339/ISO 8061
    pub expires: Timestamp,
}
