use super::*;

use std::{
    fmt,
    str::FromStr,
    time::{Duration, SystemTime},
};

use std::num::NonZeroU64;

use time::{OffsetDateTime, PrimitiveDateTime};

/**
    Snowflakes are a UUID-like system designed to embed timestamp information in a monotonic format.

    This implementation provides an EPOCH for offsetting the timestamp, and implementations for SQL/JSON interop.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Snowflake(pub NonZeroU64);

/// Arbitrarily chosen starting epoch to offset the clock by
pub const LANTERN_EPOCH: u64 = 1550102400000;

pub const LANTERN_EPOCH_ODT: OffsetDateTime = time::macros::datetime!(2019 - 02 - 14 00:00 +0);
pub const LANTERN_EPOCH_PDT: PrimitiveDateTime = time::macros::datetime!(2019 - 02 - 14 00:00);

impl Snowflake {
    #[inline]
    pub const fn null() -> Snowflake {
        Snowflake(unsafe { NonZeroU64::new_unchecked(1) })
    }

    /// Gets the number of milliseconds since the unix epoch
    #[inline]
    pub fn epoch_ms(&self) -> u64 {
        self.raw_timestamp() + LANTERN_EPOCH
    }

    #[inline]
    pub fn system_timestamp(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_millis(self.epoch_ms())
    }

    #[inline]
    pub fn timestamp(&self) -> Timestamp {
        Timestamp::UNIX_EPOCH + Duration::from_millis(self.epoch_ms())
    }

    #[inline]
    pub fn raw_timestamp(&self) -> u64 {
        self.0.get() >> 22
    }

    #[inline(always)]
    pub const fn to_u64(self) -> u64 {
        self.0.get()
    }

    #[inline(always)]
    pub const fn to_i64(self) -> i64 {
        unsafe { std::mem::transmute(self.0.get()) }
    }
}

impl FromStr for Snowflake {
    type Err = <NonZeroU64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NonZeroU64::from_str(s).map(Snowflake)
    }
}

impl fmt::Display for Snowflake {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(itoa::Buffer::new().format(self.0.get()))
    }
}

#[cfg(feature = "pg")]
mod pg_impl {
    use super::*;

    use std::error::Error;

    use bytes::BytesMut;
    use postgres_types::{accepts, to_sql_checked, FromSql, IsNull, ToSql, Type};

    impl<'a> FromSql<'a> for Snowflake {
        #[inline]
        fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
            <i64 as FromSql<'a>>::from_sql(ty, raw).map(|raw| Snowflake(NonZeroU64::new(raw as u64).unwrap()))
        }

        accepts!(INT8);
    }

    impl ToSql for Snowflake {
        #[inline]
        fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
        where
            Self: Sized,
        {
            self.to_i64().to_sql(ty, out)
        }

        accepts!(INT8);
        to_sql_checked!();
    }
}

#[cfg(feature = "rusqlite")]
mod rusqlite_impl {
    use super::*;

    use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

    impl FromSql for Snowflake {
        fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
            match value {
                ValueRef::Integer(i) if i != 0 => unsafe { Ok(Snowflake(NonZeroU64::new_unchecked(i as u64))) },
                ValueRef::Integer(_) => Err(FromSqlError::OutOfRange(0)),
                _ => Err(FromSqlError::InvalidType),
            }
        }
    }

    impl ToSql for Snowflake {
        fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
            Ok(ToSqlOutput::Owned(self.to_i64().into()))
        }
    }
}

mod serde_impl {
    use super::*;

    use std::str::FromStr;

    use serde::de::{Deserialize, Deserializer, Error, Visitor};
    use serde::ser::{Serialize, Serializer};

    impl Serialize for Snowflake {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if serializer.is_human_readable() {
                serializer.serialize_str(itoa::Buffer::new().format(self.0.get()))
            } else {
                self.0.serialize(serializer)
            }
        }
    }

    impl<'de> Deserialize<'de> for Snowflake {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            return deserializer.deserialize_any(SnowflakeVisitor);

            struct SnowflakeVisitor;

            impl<'de> Visitor<'de> for SnowflakeVisitor {
                type Value = Snowflake;

                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("a 64-bit integer or numeric string")
                }

                #[inline]
                fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
                    match NonZeroU64::new(v) {
                        Some(x) => Ok(Snowflake(x)),
                        None => Err(E::custom("expected a non-zero value")),
                    }
                }

                fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                    Snowflake::from_str(v).map_err(|e| E::custom(format!("Invalid Snowflake: {e}")))
                }
            }
        }
    }
}

#[cfg(feature = "schema")]
mod schema_impl {
    use super::Snowflake;

    use schemars::_serde_json::json;
    use schemars::{
        schema::{InstanceType, Metadata, Schema, SchemaObject, SingleOrVec},
        JsonSchema,
    };

    impl JsonSchema for Snowflake {
        fn schema_name() -> String {
            "Snowflake".to_owned()
        }

        fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> Schema {
            let mut obj = SchemaObject {
                metadata: Some(Box::new(Metadata {
                    description: Some("Snowflake (Non-Zero Integer as String)".to_owned()),
                    examples: vec![json!("354745245846793448")],
                    ..Default::default()
                })),
                instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
                ..Default::default()
            };

            // https://stackoverflow.com/a/7036407/2083075
            obj.string().pattern = Some("^[0-9]*[1-9][0-9]*$".to_owned()); // non-zero integer

            Schema::Object(obj)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sf_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<u64>(), size_of::<Option<Snowflake>>());
    }

    #[test]
    fn test_serde() {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        struct Nested {
            x: Snowflake,
        }

        let _: Snowflake = serde_json::from_str(r#""12234""#).unwrap();
        let _: Snowflake = serde_json::from_str(r#"12234"#).unwrap();
        let _: Nested = serde_json::from_str(r#"{"x": 12234}"#).unwrap();
        let _: Nested = serde_json::from_str(r#"{"x": "12234"}"#).unwrap();
    }

    #[test]
    fn test_epoch() {
        assert_eq!(LANTERN_EPOCH, LANTERN_EPOCH_ODT.unix_timestamp() as u64 * 1000);
    }
}
