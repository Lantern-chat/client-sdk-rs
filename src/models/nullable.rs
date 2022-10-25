/// Similar to `Option`, `Undefined` values can be used when data may exist but is not provided.
///
/// For example, a user biography may not be provided to any random user if they haven't
/// given permission to non-friends to view their profile, but that does not imply it doesn't exist.
///
/// Similarly, not all gateway events provide all information in objects. Again, user profiles
/// are notable in that biographies are typically excluded in events to save on bandwidth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nullable<T> {
    Undefined,
    Null,
    Some(T),
}

impl<T> From<T> for Nullable<T> {
    fn from(value: T) -> Self {
        Nullable::Some(value)
    }
}

impl<T> From<Option<T>> for Nullable<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            None => Nullable::Null,
            Some(value) => Nullable::Some(value),
        }
    }
}

impl<T> Nullable<T> {
    #[inline]
    pub const fn is_undefined(&self) -> bool {
        matches!(self, Nullable::Undefined)
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, Nullable::Null)
    }

    #[inline]
    pub const fn is_some(&self) -> bool {
        matches!(self, Nullable::Some(_))
    }

    pub fn map<F, U>(self, f: F) -> Nullable<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Nullable::Some(value) => Nullable::Some(f(value)),
            Nullable::Null => Nullable::Null,
            Nullable::Undefined => Nullable::Undefined,
        }
    }
}

impl<T> Default for Nullable<T> {
    fn default() -> Self {
        Nullable::Undefined
    }
}

mod impl_serde {
    use serde::de::{Deserialize, Deserializer, Visitor};
    use serde::ser::{Serialize, Serializer};

    use super::Nullable;

    impl<T> Serialize for Nullable<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match *self {
                Nullable::Some(ref value) => serializer.serialize_some(value),
                _ => serializer.serialize_none(),
            }
        }
    }

    impl<'de, T> Deserialize<'de> for Nullable<T>
    where
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            match Option::deserialize(deserializer) {
                Ok(None) => Ok(Nullable::Null),
                Ok(Some(value)) => Ok(Nullable::Some(value)),
                Err(e) => Err(e),
            }
        }
    }
}

#[cfg(feature = "rusqlite")]
mod rusqlite_impl {
    use super::Nullable;

    use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, Null, ToSql, ToSqlOutput, ValueRef};

    impl<T: FromSql> FromSql for Nullable<T> {
        fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
            match value {
                ValueRef::Null => Ok(Nullable::Null),
                _ => T::column_result(value).map(Nullable::Some),
            }
        }
    }

    impl<T: ToSql> ToSql for Nullable<T> {
        fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
            match self {
                Nullable::Some(val) => val.to_sql(),
                _ => Ok(ToSqlOutput::from(Null)),
            }
        }
    }
}

#[cfg(feature = "pg")]
mod pg_impl {
    use super::Nullable;

    use std::error::Error;

    use bytes::BytesMut;
    use postgres_types::{accepts, to_sql_checked, FromSql, IsNull, ToSql, Type};

    impl<'a, T: FromSql<'a>> FromSql<'a> for Nullable<T> {
        fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
            <T as FromSql>::from_sql(ty, raw).map(Nullable::Some)
        }

        #[inline]
        fn from_sql_null(_: &Type) -> Result<Self, Box<dyn Error + Sync + Send>> {
            Ok(Nullable::Null)
        }

        #[inline]
        fn accepts(ty: &Type) -> bool {
            <T as FromSql>::accepts(ty)
        }
    }

    impl<T: ToSql> ToSql for Nullable<T> {
        #[inline]
        fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
            match self {
                Nullable::Some(val) => val.to_sql(ty, out),
                _ => Ok(IsNull::Yes),
            }
        }

        #[inline]
        fn accepts(ty: &Type) -> bool {
            <T as ToSql>::accepts(ty)
        }

        to_sql_checked!();
    }
}

#[cfg(feature = "schema")]
mod schema_impl {
    use super::Nullable;

    use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};

    // TODO: Replace this with something better than Option's settings eventually.
    impl<T> JsonSchema for Nullable<T>
    where
        T: JsonSchema,
    {
        fn is_referenceable() -> bool {
            Option::<T>::is_referenceable()
        }

        fn schema_name() -> String {
            Option::<T>::schema_name()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            Option::<T>::json_schema(gen)
        }

        fn _schemars_private_non_optional_json_schema(gen: &mut SchemaGenerator) -> Schema {
            Option::<T>::_schemars_private_non_optional_json_schema(gen)
        }

        fn _schemars_private_is_option() -> bool {
            Option::<T>::_schemars_private_is_option()
        }
    }
}
