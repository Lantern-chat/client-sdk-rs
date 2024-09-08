use super::*;

use crate::models::Locale;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PartyPrefsFlags: i32 {

        const DEFAULT_FLAGS = 0;
    }
}

impl_rkyv_for_bitflags!(pub PartyPrefsFlags: i32);
impl_serde_for_bitflags!(PartyPrefsFlags);
impl_sql_for_bitflags!(PartyPrefsFlags);

impl From<u64> for PartyPrefsFlags {
    fn from(value: u64) -> Self {
        PartyPrefsFlags::from_bits_truncate(value as _)
    }
}

impl Default for PartyPrefsFlags {
    fn default() -> Self {
        Self::DEFAULT_FLAGS
    }
}

mod preferences {
    decl_newtype_prefs! {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct PartyPreferences {
    #[serde(default, skip_serializing_if = "is_default", alias = "locale")]
    pub l: Locale,
    #[serde(default, skip_serializing_if = "is_default", alias = "flags")]
    pub f: PartyPrefsFlags,
}
