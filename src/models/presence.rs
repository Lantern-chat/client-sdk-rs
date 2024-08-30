use super::*;

bitflags::bitflags! {
    /// NOTE: These flags are ordered such that larger values take precedence
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UserPresenceFlags: i16 {
        const OFFLINE   = 0;
        const AWAY      = 1 << 0;
        const MOBILE    = 1 << 1;
        const ONLINE    = 1 << 2; // ONLINE+MOBILE will be larger than ONLINE only
        const BUSY      = 1 << 3;
        const INVISIBLE = 1 << 4;
    }
}

impl_rkyv_for_bitflags!(pub UserPresenceFlags: i16);
impl_serde_for_bitflags!(UserPresenceFlags);
impl_schema_for_bitflags!(UserPresenceFlags);

impl UserPresenceFlags {
    pub const fn from_bits_truncate_public(bits: i16) -> Self {
        Self::from_bits_truncate(bits).difference(Self::INVISIBLE)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    archive(check_bytes)
)]
pub struct UserPresence {
    pub flags: UserPresenceFlags,

    /// approximately how many seconds ago they were active
    /// not present in all events or if user has disabled it
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active: Option<u64>,

    /// Updated-At timestamp as ISO-8061
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<Timestamp>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<AnyActivity>,
}

impl UserPresence {
    pub const fn new(flags: UserPresenceFlags) -> Self {
        UserPresence {
            flags,
            activity: None,
            last_active: None,
            updated_at: None,
        }
    }

    pub fn with_activty(mut self, activity: Option<AnyActivity>) -> Self {
        self.activity = activity;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    archive(check_bytes)
)]
#[serde(untagged)]
pub enum AnyActivity {
    Typed(Activity),
    // /// WARNING: Never construct this manually
    // Any(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    archive(check_bytes)
)]
pub struct Activity {}
