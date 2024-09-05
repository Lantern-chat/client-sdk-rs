/// Similar to `Option`, `Undefined` values can be used when data may exist but is not provided.
///
/// For example, a user biography may not be provided to any random user if they haven't
/// given permission to non-friends to view their profile, but that does not imply it doesn't exist.
///
/// Similarly, not all gateway events provide all information in objects. Again, user profiles
/// are notable in that biographies are typically excluded in events to save on bandwidth.
#[derive(Default, Debug, Clone, Copy, Hash)]
#[repr(u8)]
pub enum Nullable<T> {
    /// Neither present nor absent, an indeterminant value.
    #[default]
    Undefined = 0,
    /// Certainly absent of value.
    Null = 1,
    /// Certainly present of value.
    Some(T) = 2,
}

impl<T, U> PartialEq<Nullable<U>> for Nullable<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &Nullable<U>) -> bool {
        match (self, other) {
            (Nullable::Some(lhs), Nullable::Some(rhs)) => lhs.eq(rhs),
            (Nullable::Undefined, Nullable::Undefined) => true,
            (Nullable::Null, Nullable::Null) => true,
            _ => false,
        }
    }
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
    /// Returns `true` if the value is `Undefined`.
    #[inline]
    pub const fn is_undefined(&self) -> bool {
        matches!(self, Nullable::Undefined)
    }

    /// Returns `true` if the value is `Null`.
    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, Nullable::Null)
    }

    /// Returns `true` if the value is `Some`.
    #[inline]
    pub const fn is_some(&self) -> bool {
        matches!(self, Nullable::Some(_))
    }

    /// Maps an inner `Some` value to a different value,
    /// carrying over `Null` and `Undefined` unchanged.
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

    /// Converts `Nullable<T>` to `Nullable<&T>`.
    pub fn as_ref(&self) -> Nullable<&T> {
        match self {
            Nullable::Some(value) => Nullable::Some(value),
            Nullable::Null => Nullable::Null,
            Nullable::Undefined => Nullable::Undefined,
        }
    }

    /// Maps an inner `Some` value to a different value, using `Into`.
    ///
    /// Equivalent to `.map(Into::into)`.
    pub fn map_into<U>(self) -> Nullable<U>
    where
        T: Into<U>,
    {
        self.map(Into::into)
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
                Nullable::Undefined => {
                    panic!("Cannot serialize an `Undefined` value, use skip_serializing_if = \"Nullable::is_undefined\"")
                }
                Nullable::Null => serializer.serialize_none(),
                Nullable::Some(ref value) => serializer.serialize_some(value),
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

#[cfg(feature = "rkyv")]
mod rkyv_impl {
    use std::{marker::PhantomData, mem::MaybeUninit};

    use super::*;

    use rkyv::{
        bytecheck::{CheckBytes, InvalidEnumDiscriminantError, UnnamedEnumVariantCheckContext},
        rancor::{Fallible, Source, Trace},
        traits::CopyOptimization,
        Archive, Archived, Deserialize, Place, Portable, Serialize,
    };

    unsafe impl<T> Portable for Nullable<T> where T: Portable {}

    #[repr(u8)]
    pub enum ArchivedNullableTag {
        Undefined = 0,
        Null = 1,
        Some = 2,
    }

    #[repr(C)]
    struct NullableRepr<T>(ArchivedNullableTag, T, PhantomData<Nullable<T>>);

    impl<T: Archive> Archive for Nullable<T> {
        type Archived = Nullable<T::Archived>;
        type Resolver = Nullable<T::Resolver>;

        const COPY_OPTIMIZATION: CopyOptimization<Self> =
            unsafe { CopyOptimization::enable_if(<T as Archive>::COPY_OPTIMIZATION.is_enabled()) };

        #[inline]
        fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
            match resolver {
                Nullable::Undefined => unsafe {
                    out.cast_unchecked::<ArchivedNullableTag>().write_unchecked(ArchivedNullableTag::Undefined);
                },
                Nullable::Null => unsafe {
                    out.cast_unchecked::<ArchivedNullableTag>().write_unchecked(ArchivedNullableTag::Null);
                },
                Nullable::Some(resolver) => unsafe {
                    let out = out.cast_unchecked::<NullableRepr<Archived<T>>>();

                    let Nullable::Some(ref value) = *self else {
                        core::hint::unreachable_unchecked()
                    };

                    core::ptr::addr_of_mut!((*out.ptr()).0).write(ArchivedNullableTag::Some);

                    <T as Archive>::resolve(
                        value,
                        resolver,
                        Place::from_field_unchecked(out, core::ptr::addr_of_mut!((*out.ptr()).1)),
                    );
                },
            }
        }
    }

    impl<T: Serialize<S>, S: Fallible + ?Sized> Serialize<S> for Nullable<T> {
        #[inline]
        fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            match self {
                Nullable::Undefined => Ok(Nullable::Undefined),
                Nullable::Null => Ok(Nullable::Null),
                Nullable::Some(value) => Ok(Nullable::Some(value.serialize(serializer)?)),
            }
        }
    }

    impl<T: Archive, D: Fallible + ?Sized> Deserialize<Nullable<T>, D> for Nullable<T::Archived>
    where
        Archived<T>: Deserialize<T, D>,
    {
        #[inline]
        fn deserialize(&self, deserializer: &mut D) -> Result<Nullable<T>, D::Error> {
            match self {
                Nullable::Undefined => Ok(Nullable::Undefined),
                Nullable::Null => Ok(Nullable::Null),
                Nullable::Some(value) => Ok(Nullable::Some(value.deserialize(deserializer)?)),
            }
        }
    }

    unsafe impl<T, C> CheckBytes<C> for Nullable<T>
    where
        C: Fallible + ?Sized,
        <C as Fallible>::Error: Source,
        T: CheckBytes<C>,
    {
        unsafe fn check_bytes(value: *const Self, ctx: &mut C) -> Result<(), C::Error> {
            let tag = *value.cast::<u8>();

            match tag {
                0 => Ok(()),
                1 => Ok(()),
                2 => {
                    let value = value.cast::<NullableRepr<T>>();

                    <T as CheckBytes<C>>::check_bytes(core::ptr::addr_of!((*value).1), ctx).map_err(|e| {
                        <<C as Fallible>::Error as Trace>::trace(
                            e,
                            UnnamedEnumVariantCheckContext {
                                enum_name: "Nullable",
                                variant_name: "Some",
                                field_index: 1,
                            },
                        )
                    })
                }
                invalid_discriminant => Err(<<C as Fallible>::Error as Source>::new(InvalidEnumDiscriminantError {
                    enum_name: "Nullable",
                    invalid_discriminant,
                })),
            }
        }
    }
}
