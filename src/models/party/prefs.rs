use std::collections::HashMap;
use std::fmt;

use serde_json::Value;

use super::*;

use crate::models::Locale;

bitflags::bitflags! {
    pub struct PartyPrefsFlags: i32 {

        const DEFAULT_FLAGS = 0;
    }
}

common::impl_serde_for_bitflags!(PartyPrefsFlags);
common::impl_sql_for_bitflags!(PartyPrefsFlags);

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PartyPreference {
    Locale,

    Flags,

    #[serde(other)]
    InvalidField,
}

impl fmt::Display for PartyPreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use serde::Serialize;
        self.serialize(f)
    }
}

use crate::util::prefs::*;

pub type PartyPreferences = PreferenceMap<PartyPreference, Hasher>;
pub type PartyPreferenceError = PreferenceError<PartyPreference>;

impl Preference for PartyPreference {
    type Flags = PartyPrefsFlags;

    const FLAGS_KEY: Self = PartyPreference::Flags;

    fn validate(&self, _value: &Value) -> Result<(), PartyPreferenceError> {
        Ok(())
    }

    fn is_default(&self, _value: &Value, _flags: PartyPrefsFlags) -> bool {
        false
    }
}
