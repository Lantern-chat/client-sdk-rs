use std::ops::Deref;

use super::*;

mod prefs;
pub use prefs::*;

bitflags::bitflags! {
    #[derive(Default)]
    pub struct PartyFlags: i32 {
        /// Must have a verified email address
        const EMAIL         = 1 << 0;
        /// Must have a verified phone number
        const PHONE         = 1 << 1;
        /// Must be a Lantern user for longer than 5 minutes
        const NEW_USER      = 1 << 2;
        /// Must be a member of the server for longer than 10 minutes
        const NEW_MEMBER    = 1 << 3;
        /// Must have MFA enabled
        const MFA_ENABLED   = 1 << 4;

        /// Party is marked as "adult"
        ///
        /// This affects viewing on iOS apps and
        /// the minimum age required to join.
        const ADULT         = 1 << 5;

        /// Top 6 bits are a language code
        const LANGUAGE = 0b11_11_11 << (32 - 6);

        const SECURITY = 0
            | Self::EMAIL.bits
            | Self::PHONE.bits
            | Self::NEW_USER.bits
            | Self::NEW_MEMBER.bits
            | Self::MFA_ENABLED.bits;
    }
}

common::impl_serde_for_bitflags!(PartyFlags);
common::impl_schema_for_bitflags!(PartyFlags);
common::impl_sql_for_bitflags!(PartyFlags);

//#[derive(Debug, Clone, Serialize, Deserialize)]
//#[serde(untagged)]
//pub enum UnavailableParty {
//    Available(Party),
//    Unavailable { id: Snowflake, unavailable: bool },
//}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct PartialParty {
    pub id: Snowflake,

    /// Party name
    pub name: SmolStr,

    /// Description of the party, if publicly listed
    pub description: Option<SmolStr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct Party {
    #[serde(flatten)]
    pub partial: PartialParty,

    pub flags: PartyFlags,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<SmolStr>,

    #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
    pub banner: Nullable<SmolStr>,

    pub default_room: Snowflake,

    /// Position of party is user's party list, will be null if not joined
    #[serde(default)]
    pub position: Option<i16>,

    /// Id of owner user
    pub owner: Snowflake,

    pub roles: ThinVec<Role>,

    pub emotes: ThinVec<Emote>,

    pub pin_folders: ThinVec<PinFolder>,
}

impl Deref for Party {
    type Target = PartialParty;

    fn deref(&self) -> &Self::Target {
        &self.partial
    }
}

bitflags::bitflags! {
    pub struct PartyMemberFlags: i16 {
        const BANNED = 1 << 0;
    }
}

common::impl_serde_for_bitflags!(PartyMemberFlags);
common::impl_schema_for_bitflags!(PartyMemberFlags);
common::impl_sql_for_bitflags!(PartyMemberFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct PartialPartyMember {
    /// Will be `None` if no longer in party
    pub joined_at: Option<Timestamp>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<PartyMemberFlags>,

    /// List of Role id snowflakes
    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub roles: Option<ThinVec<Snowflake>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct PartyMember {
    pub user: User,

    #[serde(flatten)]
    pub partial: PartialPartyMember,
}

impl Deref for PartyMember {
    type Target = PartialPartyMember;

    fn deref(&self) -> &Self::Target {
        &self.partial
    }
}

bitflags::bitflags! {
    pub struct PinFolderFlags: i32 {
        const COLOR = 0x00_FF_FF_FFu32 as i32; // top 24 bits
    }
}

common::impl_serde_for_bitflags!(PinFolderFlags);
common::impl_schema_for_bitflags!(PinFolderFlags);
common::impl_sql_for_bitflags!(PinFolderFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct PinFolder {
    pub id: Snowflake,
    pub name: SmolStr,
    pub flags: PinFolderFlags,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<SmolStr>,
}
