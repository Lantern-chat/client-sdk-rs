macro_rules! impl_rkyv_for_bitflags {
    ($vis:vis $name:ident: $ty:ty) => {
        #[cfg(feature = "rkyv")]
        rkyv_rpc::bitflags!(@RKYV_ONLY $vis $name: $ty);
    }
}

#[cfg(feature = "rkyv")]
macro_rules! enum_codes {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $archived_vis:vis $repr:ty $(= $unknown:ident)? {
            $($(#[$variant_meta:meta])* $code:literal = $variant:ident,)*
        }
    ) => {
        rkyv_rpc::enum_codes! {
            $(#[$meta])*
            $vis enum $name: $archived_vis $repr $(= $unknown)? {
                $($(#[$variant_meta])* $code = $variant,)*
            }
        }
    };
}

#[cfg(not(feature = "rkyv"))]
macro_rules! enum_codes {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $archived_vis:vis $repr:ty $(= $unknown:ident)? {
            $($(#[$variant_meta:meta])* $code:literal = $variant:ident,)*
        }
    ) => {
        $(#[$meta])*
        #[repr($repr)]
        $vis enum $name {
            $($(#[$variant_meta])* $variant = $code,)*
        }
    };
}

// NOTE: Passing `$repr` as ident is required for it to be matched against u8/i8 specialization. I don't know why.

#[cfg(feature = "rkyv")]
macro_rules! decl_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $repr:ident  {
            $($(#[$variant_meta:meta])* $code:literal = $variant:ident,)* $(,)?
        }
    ) => {
        rkyv_rpc::unit_enum! {
            $(#[$meta])*
            $vis enum $name: $repr {
                $($(#[$variant_meta])* $code = $variant,)*
            }
        }
    };
}

#[cfg(not(feature = "rkyv"))]
macro_rules! decl_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $repr:ident {
            $($(#[$variant_meta:meta])* $code:literal = $variant:ident,)* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr($repr)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $($(#[$variant_meta])* $variant = $code,)*
        }
    };
}

#[allow(unused)]
macro_rules! impl_sql_common {
    (@ACCEPTS $id:ident) => {
        fn accepts(ty: &postgres_types::Type) -> bool {
            use postgres_types::Type;

            let target = const {
                match core::mem::size_of::<$id>() {
                    1 | 2 => Some(Type::INT2),
                    4 => Some(Type::INT4),
                    8 => Some(Type::INT8),
                    _ => None
                }
            };

            matches!(target, Some(target) if *ty == target)
        }
    };
}

macro_rules! impl_sql_for_bitflags {
    ($id:ident) => {
        #[cfg(feature = "rusqlite")]
        const _: () = {
            use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

            impl FromSql for $id {
                fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                    match value {
                        ValueRef::Integer(value) => Ok(Self::from_bits_truncate(value as _)),
                        _ => Err(FromSqlError::InvalidType),
                    }
                }
            }

            impl ToSql for $id {
                fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
                    Ok(ToSqlOutput::Owned(self.bits().into()))
                }
            }
        };

        #[cfg(feature = "pg")]
        const _: () = {
            use std::error::Error;

            use bytes::BytesMut;
            use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};

            impl<'a> FromSql<'a> for $id {
                #[inline]
                fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
                    Ok(Self::from_bits_truncate(FromSql::from_sql(ty, raw)?))
                }

                impl_sql_common!(@ACCEPTS $id);
            }

            impl ToSql for $id {
                #[inline]
                fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
                where
                    Self: Sized,
                {
                    self.bits().to_sql(ty, out)
                }

                impl_sql_common!(@ACCEPTS $id);
                to_sql_checked!();
            }

            #[cfg(feature = "rkyv")]
            const _: () = {
                paste::paste! {
                    // NOTE: Cannot use `Archived<$id>` as it triggers a trait conflict bug in rustc,
                    // luckily we use the default naming convention for archived types.
                    impl ToSql for [<Archived $id>] {
                        #[inline]
                        fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
                        where
                            Self: Sized,
                        {
                            self.to_native().to_sql(ty, out)
                        }

                        impl_sql_common!(@ACCEPTS $id);
                        to_sql_checked!();
                    }
                }
            };
        };
    };
}

macro_rules! impl_sql_for_enum_primitive {
    ($id:ident) => {
        #[cfg(feature = "rusqlite")]
        const _: () = {
            use num_traits::{FromPrimitive, ToPrimitive};
            use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

            impl FromSql for $id {
                fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                    match value {
                        ValueRef::Integer(value) => FromPrimitive::from_i64(value).ok_or_else(|| FromSqlError::OutOfRange(value)),
                        _ => Err(FromSqlError::InvalidType),
                    }
                }
            }

            impl ToSql for $id {
                fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
                    Ok(ToSqlOutput::Owned(self.to_i64().into()))
                }
            }
        };

        #[cfg(feature = "pg")]
        const _: () = {
            use num_traits::{FromPrimitive, ToPrimitive};
            use std::error::Error;

            use bytes::BytesMut;
            use postgres_types::{accepts, to_sql_checked, FromSql, IsNull, ToSql, Type};

            #[derive(Debug, Clone, Copy)]
            struct FromSqlError(i64);

            impl std::fmt::Display for FromSqlError {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use std::fmt::Write;

                    write!(
                        f,
                        concat!("Unable to convert {} to type '", stringify!($id), "'"),
                        self.0
                    )
                }
            }

            impl std::error::Error for FromSqlError {}

            impl<'a> FromSql<'a> for $id {
                #[inline]
                fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
                    let raw_value = match ty {
                        &Type::INT2 => <i16 as FromSql>::from_sql(ty, raw)? as i64,
                        &Type::INT4 => <i32 as FromSql>::from_sql(ty, raw)? as i64,
                        &Type::INT8 => <i64 as FromSql>::from_sql(ty, raw)? as i64,
                        _ => unreachable!(),
                    };

                    <$id as FromPrimitive>::from_i64(raw_value)
                        .ok_or_else(|| Box::new(FromSqlError(raw_value)) as Box<dyn Error + Sync + Send>)
                }

                accepts!(INT2, INT4, INT8);
            }

            impl ToSql for $id {
                #[inline]
                fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
                where
                    Self: Sized,
                {
                    match std::mem::size_of::<Self>() {
                        1 | 2 => self.to_i16().to_sql(ty, out),
                        4 => self.to_i32().to_sql(ty, out),
                        8 => self.to_i64().to_sql(ty, out),
                        _ => unreachable!(),
                    }
                }

                impl_sql_common!(@ACCEPTS $id);
                to_sql_checked!();
            }
        };
    };
}

macro_rules! impl_schema_for_bitflags {
    ($name:ident) => {
        #[cfg(feature = "schemars")]
        const _: () = {
            use schemars::_serde_json::json;
            use schemars::{
                schema::{InstanceType, Metadata, Schema, SchemaObject, SingleOrVec},
                JsonSchema,
            };

            impl JsonSchema for $name {
                fn schema_name() -> String {
                    stringify!($name).to_owned()
                }

                fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> Schema {
                    let size = std::mem::size_of::<Self>();
                    let bignum = size >= 16;

                    let mut obj = SchemaObject {
                        metadata: Some(Box::new(Metadata {
                            description: Some(format!("{} Bitflags", stringify!($name))),
                            examples: vec![json!($name::all().bits())],
                            ..Default::default()
                        })),
                        instance_type: Some(SingleOrVec::Single(Box::new(if bignum {
                            InstanceType::String
                        } else {
                            InstanceType::Number
                        }))),
                        ..Default::default()
                    };

                    if bignum {
                        obj.string().pattern = Some(match size {
                            16 => "\\d{0,38}".to_owned(),
                            _ => unreachable!(),
                        });
                    } else {
                        obj.number().maximum = Some($name::all().bits() as _);
                    }

                    Schema::Object(obj)
                }
            }
        };
    };
}
