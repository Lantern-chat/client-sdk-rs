use core::num::NonZeroU32;
use num_traits::FromPrimitive;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_primitive_derive::Primitive)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
#[repr(u8)]
pub enum RoomKind {
    Text = 0,
    DirectMessage = 1,
    GroupMessage = 2,
    Voice = 3,
    UserForum = 4,
    // max value cannot exceed 15
}

bitflags2! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RoomFlags: i16 {
        const KIND    = 0xF; // first four bits are the kind
        const NSFW    = 1 << 4;
        const DEFAULT = 1 << 5;
    }
}

impl_rkyv_for_bitflags!(pub RoomFlags: i16);
impl_serde_for_bitflags!(RoomFlags);
impl_schema_for_bitflags!(RoomFlags);
impl_sql_for_bitflags!(RoomFlags);

impl RoomFlags {
    #[must_use]
    pub fn kind(self) -> RoomKind {
        // all rooms derive from the text room, so basic queries
        // will still function if the SDK is not updated as it should
        RoomKind::from_i16(self.bits() & 0xF).unwrap_or(RoomKind::Text)
    }
}

impl From<RoomKind> for RoomFlags {
    fn from(value: RoomKind) -> Self {
        RoomFlags::from_bits_retain(value as u8 as i16)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct Room {
    pub id: RoomId,

    pub flags: RoomFlags,

    pub party_id: PartyId,

    pub avatar: Option<EncryptedSnowflake>,

    pub name: SmolStr,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<SmolStr>,

    /// Sort order
    pub position: i16,

    /// Slow-mode rate limit, in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "rkyv", rkyv(with = Niche))]
    pub rate_limit_per_user: Option<NonZeroU32>,

    /// Parent room ID for categories
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "rkyv", rkyv(with = NicheSnowflake))]
    pub parent_id: Option<RoomId>,

    /// Permission overwrites for this room
    #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
    pub overwrites: ThinVec<Overwrite>,
    // /// Direct/Group Message Users
    // #[serde(default, skip_serializing_if = "Vec::is_empty")]
    // pub recipients: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct FullRoom {
    #[serde(flatten)]
    pub room: Room,

    pub perms: Permissions,
}
