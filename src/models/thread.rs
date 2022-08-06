use super::*;

bitflags::bitflags! {
    pub struct ThreadFlags: i16 {
        /// Forum-style thread
        const FORUM   = 1 << 0;
    }
}

impl_serde_for_bitflags!(ThreadFlags);
impl_schema_for_bitflags!(ThreadFlags);
impl_pg_for_bitflags!(ThreadFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Thread {
    pub id: Snowflake,
    pub parent: Message,
    pub flags: ThreadFlags,
}
