use std::ops::Deref;

use super::*;

bitflags::bitflags! {
    pub struct PartyFlags: i16 {
        /// Top 6 bits are a language code
        const LANGUAGE = 0b111_111 << (16 - 6);
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
//pub enum UnvailableParty {
//    Available(Party),
//    Unavailable { id: Snowflake, unavailable: bool },
//}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Party {
    #[serde(flatten)]
    pub partial: PartialParty,

    /// Id of owner user
    pub owner: Snowflake,

    pub security: SecurityFlags,

    pub roles: Vec<Role>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emotes: Vec<Emote>,

    pub avatar: Option<SmolStr>,

    #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
    pub banner: Nullable<SmolStr>,

    pub position: i16,

    pub default_room: Snowflake,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PartialParty {
    pub id: Snowflake,

    /// Party name
    pub name: SmolStr,

    /// Discription of the party, if publicly listed
    pub description: Option<SmolStr>,
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
pub struct PartyMember {
    /// User information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<PartyMemberFlags>,

    /// List of Role id snowflakes
    #[serde(default, skip_serializing_if = "is_none_or_empty")]
    pub roles: Option<Vec<Snowflake>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence: Option<UserPresence>,
}
