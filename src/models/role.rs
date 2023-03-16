use super::*;

bitflags::bitflags! {
    #[derive(Default)]
    pub struct RoleFlags: i16 {
        const HOIST         = 1 << 0;
        const MENTIONABLE   = 1 << 1;
    }
}

serde_shims::impl_serde_for_bitflags!(RoleFlags);
impl_schema_for_bitflags!(RoleFlags);
impl_sql_for_bitflags!(RoleFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Role {
    pub id: Snowflake,

    // TODO: Revist removing this
    pub party_id: Snowflake,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<SmolStr>,
    pub name: SmolStr,
    pub permissions: Permissions,
    pub color: Option<u32>,
    pub position: i16,
    pub flags: RoleFlags,
}

impl Role {
    pub fn is_mentionable(&self) -> bool {
        self.flags.contains(RoleFlags::MENTIONABLE)
    }

    pub fn is_admin(&self) -> bool {
        self.permissions.contains(Permissions::ADMINISTRATOR)
    }
}
