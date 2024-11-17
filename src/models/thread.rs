use super::*;

bitflags2! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ThreadFlags: i16 {
        /// Forum-style thread
        const FORUM   = 1 << 0;
    }
}

impl_rkyv_for_bitflags!(pub ThreadFlags: i16);
impl_serde_for_bitflags!(ThreadFlags);
impl_schema_for_bitflags!(ThreadFlags);
impl_sql_for_bitflags!(ThreadFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct Thread {
    pub id: ThreadId,
    pub parent: Message,
    pub flags: ThreadFlags,
}
