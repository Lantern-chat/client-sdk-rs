use core::ops::Deref;

use super::*;

mod prefs;
pub use prefs::*;

bitflags2! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

        /// Another way to refer to a direct-message is a "closed" party.
        const CLOSED        = 1 << 6;

        /// Top 6 bits are a language code
        const LANGUAGE = 0b11_11_11 << (32 - 6);

        /// Combination of all security flags: EMAIL, PHONE, NEW_USER, NEW_MEMBER, MFA_ENABLED
        const SECURITY = 0
            | Self::EMAIL.bits()
            | Self::PHONE.bits()
            | Self::NEW_USER.bits()
            | Self::NEW_MEMBER.bits()
            | Self::MFA_ENABLED.bits();
    }
}

impl_rkyv_for_bitflags!(pub PartyFlags: i32);
impl_serde_for_bitflags!(PartyFlags);
impl_schema_for_bitflags!(PartyFlags);
impl_sql_for_bitflags!(PartyFlags);

//#[derive(Debug, Clone, Serialize, Deserialize)]
//#[serde(untagged)]
//pub enum UnavailableParty {
//    Available(Party),
//    Unavailable { id: PartyId, unavailable: bool },
//}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct PartialParty {
    pub id: PartyId,

    /// Party name
    pub name: SmolStr,

    /// Description of the party, if publicly listed
    pub description: Option<ThinString>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct Party {
    #[serde(flatten)]
    pub partial: PartialParty,

    pub flags: PartyFlags,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<EncryptedSnowflake>,

    #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
    pub banner: Nullable<EncryptedSnowflake>,

    pub default_room: RoomId,

    /// Position of party is user's party list, will be null if not joined
    #[serde(default)]
    pub position: Option<i16>,

    /// Id of owner user
    pub owner: UserId,

    pub roles: ThinVec<Role>,

    pub emotes: ThinVec<Emote>,

    pub folders: ThinVec<PinFolder>,
}

impl Deref for Party {
    type Target = PartialParty;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.partial
    }
}

#[cfg(feature = "rkyv")]
impl Deref for ArchivedParty {
    type Target = ArchivedPartialParty;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.partial
    }
}

bitflags2! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PartyMemberFlags: i16 {
        const BANNED = 1 << 0;
    }
}

impl Default for PartyMemberFlags {
    fn default() -> Self {
        PartyMemberFlags::empty()
    }
}

impl_rkyv_for_bitflags!(pub PartyMemberFlags: i16);
impl_serde_for_bitflags!(PartyMemberFlags);
impl_schema_for_bitflags!(PartyMemberFlags);
impl_sql_for_bitflags!(PartyMemberFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct PartyMember {
    pub user: User,

    /// Will be `None` if no longer in party
    pub joined_at: Option<Timestamp>,

    #[serde(default, skip_serializing_if = "PartyMemberFlags::is_empty")]
    pub flags: PartyMemberFlags,

    /// List of Role id snowflakes, may be excluded from some queries
    #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
    pub roles: ThinVec<RoleId>,
}

impl Deref for PartyMember {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

bitflags2! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PinFolderFlags: i32 {
        const COLOR = 0x00_FF_FF_FFu32 as i32; // top 24 bits
    }
}

impl_rkyv_for_bitflags!(pub PinFolderFlags: i32);
impl_serde_for_bitflags!(PinFolderFlags);
impl_schema_for_bitflags!(PinFolderFlags);
impl_sql_for_bitflags!(PinFolderFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct PinFolder {
    pub id: FolderId,
    pub name: SmolStr,
    pub flags: PinFolderFlags,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<SmolStr>,
}
