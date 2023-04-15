use std::ops::Deref;

use super::*;

bitflags::bitflags! {
    pub struct PartyFlags: i16 {
        /// Top 6 bits are a language code
        const LANGUAGE = 0b111111 << (16 - 6);
    }

    #[derive(Default)]
    pub struct SecurityFlags: i16 {
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
    }
}

serde_shims::impl_serde_for_bitflags!(SecurityFlags);
impl_schema_for_bitflags!(SecurityFlags);
impl_sql_for_bitflags!(SecurityFlags);

//#[derive(Debug, Clone, Serialize, Deserialize)]
//#[serde(untagged)]
//pub enum UnavailableParty {
//    Available(Party),
//    Unavailable { id: Snowflake, unavailable: bool },
//}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PartialParty {
    pub id: Snowflake,

    /// Party name
    pub name: SmolStr,

    /// Description of the party, if publicly listed
    pub description: Option<SmolStr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Party {
    #[serde(flatten)]
    pub partial: PartialParty,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<SmolStr>,

    #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
    pub banner: Nullable<SmolStr>,

    pub default_room: Snowflake,

    /// Position of party is user's party list, will be null if not joined
    #[serde(default)]
    pub position: Option<i16>,

    pub security: SecurityFlags,

    /// Id of owner user
    pub owner: Snowflake,

    pub roles: Vec<Role>,

    pub emotes: Vec<Emote>,

    pub pin_folders: Vec<PinFolder>,
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

serde_shims::impl_serde_for_bitflags!(PartyMemberFlags);
impl_schema_for_bitflags!(PartyMemberFlags);
impl_sql_for_bitflags!(PartyMemberFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PartialPartyMember {
    /// Will be `None` if no longer in party
    pub joined_at: Option<Timestamp>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<PartyMemberFlags>,

    /// List of Role id snowflakes
    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub roles: Option<Vec<Snowflake>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

serde_shims::impl_serde_for_bitflags!(PinFolderFlags);
impl_schema_for_bitflags!(PinFolderFlags);
impl_sql_for_bitflags!(PinFolderFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PinFolder {
    pub id: Snowflake,
    pub name: SmolStr,
    pub flags: PinFolderFlags,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<SmolStr>,
}
