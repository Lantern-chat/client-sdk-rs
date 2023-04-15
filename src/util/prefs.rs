use std::collections::hash_map::{HashMap, RandomState};
use std::{fmt, hash};

use serde_json::Value;

#[derive(Debug, Clone, Copy)]
pub struct PreferenceError<P> {
    pub field: P,
    pub kind: PreferenceErrorKind,
}

impl<P: fmt::Display> fmt::Display for PreferenceError<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self.kind {
            PreferenceErrorKind::InvalidType => "is invalid type",
            PreferenceErrorKind::InvalidValue => "has an invalid value",
        };
        write!(f, "Preference Error: \"{}\" {}", self.field, kind)
    }
}

impl<P: fmt::Debug + fmt::Display> std::error::Error for PreferenceError<P> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferenceErrorKind {
    InvalidType,
    InvalidValue,
}

use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PreferenceMap<P: Preference, H: Default + hash::BuildHasher = RandomState>(
    #[serde(bound(serialize = "P: Serialize", deserialize = "P: DeserializeOwned"))] HashMap<P, Value, H>,
);

#[cfg(feature = "schema")]
impl<P: Preference, H: Default + hash::BuildHasher> schemars::JsonSchema for PreferenceMap<P, H> {
    fn schema_name() -> std::string::String {
        "PreferenceMap".to_owned()
    }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<HashMap<P, Value, H>>()
    }
}

pub trait Preference: Sized + hash::Hash + Eq {
    type Flags: Copy + Default + From<u64>;

    const FLAGS_KEY: Self;

    fn validate(&self, value: &Value) -> Result<(), PreferenceError<Self>>;
    fn is_default(&self, value: &Value, flags: Self::Flags) -> bool;
}

impl<P: Preference, H: Default + hash::BuildHasher> PreferenceMap<P, H> {
    /// Checks if there are any invalid preference fields
    pub fn validate(&self) -> Result<(), PreferenceError<P>> {
        for (field, value) in self.0.iter() {
            field.validate(value)?;
        }

        Ok(())
    }

    /// Removes any invalid preference fields
    pub fn clean(&mut self) {
        self.0.retain(|field, value| field.validate(value).is_ok())
    }

    /// Retreives the `flags` field
    pub fn flags(&self) -> P::Flags {
        match self.0.get(&P::FLAGS_KEY).and_then(Value::as_u64) {
            Some(value) => From::from(value as _),
            None => Default::default(),
        }
    }

    /// If a field is the default value, set it to `null`
    pub fn nullify_defaults(&mut self) {
        let flags = self.flags();

        for (field, value) in self.0.iter_mut() {
            if field.is_default(value, flags) {
                *value = Value::Null;
            }
        }
    }

    pub fn merge(&mut self, new: &mut Self) {
        for (field, value) in new.0.drain() {
            self.0.insert(field, value);
        }
    }
}
