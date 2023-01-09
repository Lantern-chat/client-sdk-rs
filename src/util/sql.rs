#[allow(unused)]
macro_rules! impl_sql_common {
    (@ACCEPTS $id:ident) => {
        fn accepts(ty: &postgres_types::Type) -> bool {
            use postgres_types::Type;

            *ty == match std::mem::size_of::<$id>() {
                1 | 2 => Type::INT2,
                4 => Type::INT4,
                8 => Type::INT8,
                _ => return false,
            }
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
                        ValueRef::Integer(value) => {
                            FromPrimitive::from_i64(value).ok_or_else(|| FromSqlError::OutOfRange(value))
                        }
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
