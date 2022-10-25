macro_rules! impl_sql_for_bitflags {
    (@ACCEPTS $id:ident) => {
        fn accepts(ty: &postgres_types::Type) -> bool {
            use postgres_types::Type;

            *ty == match std::mem::size_of::<$id>() {
                1 => Type::CHAR,
                2 => Type::INT2,
                4 => Type::INT4,
                8 => Type::INT8,
                _ => return false,
            }
        }
    };
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

                impl_sql_for_bitflags!(@ACCEPTS $id);
            }

            impl ToSql for $id {
                #[inline]
                fn to_sql(
                    &self,
                    ty: &Type,
                    out: &mut BytesMut,
                ) -> Result<IsNull, Box<dyn Error + Sync + Send>>
                where
                    Self: Sized,
                {
                    self.bits().to_sql(ty, out)
                }

                impl_sql_for_bitflags!(@ACCEPTS $id);
                to_sql_checked!();
            }
        };
    };
}
