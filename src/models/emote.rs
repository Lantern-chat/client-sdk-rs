use super::*;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct EmoteFlags: i16 {
        const ANIMATED = 1 << 0;
        const STICKER  = 1 << 1;
        const NSFW     = 1 << 2;
    }
}

common::impl_serde_for_bitflags!(EmoteFlags);
common::impl_schema_for_bitflags!(EmoteFlags);
common::impl_sql_for_bitflags!(EmoteFlags);

// TODO: Add inline preview?
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct CustomEmote {
    pub id: EmoteId,
    pub party_id: PartyId,
    pub asset: FileId,
    pub name: SmolStr,
    pub flags: EmoteFlags,
    pub aspect_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
#[serde(untagged)]
pub enum Emote {
    Emoji { emoji: SmolStr },
    Custom(CustomEmote),
}
