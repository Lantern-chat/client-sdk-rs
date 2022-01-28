use super::*;

bitflags::bitflags! {
    pub struct ThreadFlags: i16 {
        /// Forum-style thread
        const FORUM   = 1 << 0;
    }
}

serde_shims::impl_serde_for_bitflags!(ThreadFlags);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: Snowflake,
    pub parent: Message,
    pub flags: ThreadFlags,
}
