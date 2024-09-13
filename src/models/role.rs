use super::*;

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RoleFlags: i16 {
        const HOIST         = 1 << 0;
        const MENTIONABLE   = 1 << 1;
    }
}

impl_rkyv_for_bitflags!(pub RoleFlags: i16);
impl_serde_for_bitflags!(RoleFlags);
impl_schema_for_bitflags!(RoleFlags);
impl_sql_for_bitflags!(RoleFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct Role {
    pub id: RoleId,

    // TODO: Revist removing this
    pub party_id: PartyId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<EncryptedSnowflake>,
    pub name: SmolStr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<SmolStr>,
    pub permissions: Permissions,
    pub color: Option<u32>, // can be intentionally null
    pub position: i16,
    pub flags: RoleFlags,
}

impl Role {
    #[must_use]
    pub fn is_mentionable(&self) -> bool {
        self.flags.contains(RoleFlags::MENTIONABLE)
    }

    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.permissions.contains(Permissions::ADMINISTRATOR)
    }
}
