use num_traits::FromPrimitive;
use std::num::NonZeroU32;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_primitive_derive::Primitive)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    archive(check_bytes, copy_safe)
)]
#[repr(u8)]
pub enum RoomKind {
    Text = 0,
    DirectMessage = 1,
    GroupMessage = 2,
    Voice = 3,
    UserForum = 4,
    // max value cannot exceed 15
}

bitflags::bitflags! {
    pub struct RoomFlags: i16 {
        const KIND    = 0xF; // first four bits are the kind
        const NSFW    = 1 << 4;
        const DEFAULT = 1 << 5;
    }
}

common::impl_serde_for_bitflags!(RoomFlags);
common::impl_schema_for_bitflags!(RoomFlags);
common::impl_sql_for_bitflags!(RoomFlags);

impl RoomFlags {
    pub fn kind(self) -> RoomKind {
        // all rooms derive from the text room, so basic queries
        // will still function if the SDK is not updated as it should
        RoomKind::from_i16(self.bits & 0xF).unwrap_or(RoomKind::Text)
    }
}

impl From<RoomKind> for RoomFlags {
    fn from(value: RoomKind) -> Self {
        unsafe { RoomFlags::from_bits_unchecked(value as u8 as i16) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct Room {
    pub id: Snowflake,

    pub flags: RoomFlags,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "rkyv", with(NicheSnowflake))]
    pub party_id: Option<Snowflake>,

    pub avatar: Option<SmolStr>,

    pub name: SmolStr,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic: Option<SmolStr>,

    /// Sort order
    pub position: i16,

    /// Slow-mode rate limit, in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "rkyv", with(Niche))]
    pub rate_limit_per_user: Option<NonZeroU32>,

    /// Parent room ID for categories
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "rkyv", with(NicheSnowflake))]
    pub parent_id: Option<Snowflake>,

    /// Permission overwrites for this room
    #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
    #[cfg_attr(feature = "rkyv", with(rkyv::with::CopyOptimize))]
    pub overwrites: ThinVec<Overwrite>,
    // /// Direct/Group Message Users
    // #[serde(default, skip_serializing_if = "Vec::is_empty")]
    // pub recipients: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct FullRoom {
    #[serde(flatten)]
    pub room: Room,

    pub perms: Permissions,
}
