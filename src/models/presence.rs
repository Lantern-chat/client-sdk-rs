use super::*;

bitflags::bitflags! {
    /// NOTE: These flags are ordered such that larger values take precedence
    pub struct UserPresenceFlags: i16 {
        const OFFLINE   = 0;
        const AWAY      = 1 << 0;
        const MOBILE    = 1 << 1;
        const ONLINE    = 1 << 2; // ONLINE+MOBILE will be larger than ONLINE only
        const BUSY      = 1 << 3;
        const INVISIBLE = 1 << 4;
    }
}

serde_shims::impl_serde_for_bitflags!(UserPresenceFlags);
impl_schema_for_bitflags!(UserPresenceFlags);
impl_pg_for_bitflags!(UserPresenceFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UserPresence {
    pub flags: UserPresenceFlags,

    /// Updated-At timestamp as ISO-8061
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<Timestamp>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<AnyActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum AnyActivity {
    Typed(Activity),

    /// WARNING: Never construct this manually
    Any(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Activity {}
