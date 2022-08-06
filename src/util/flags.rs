macro_rules! impl_pg_for_bitflags {
    (@ACCEPTS $id:ident) => {
        fn accepts(ty: &postgres_types::Type) -> bool {
            use postgres_types::Type;

            *ty == match std::mem::size_of_val(&$id::empty()) {
                1 => Type::CHAR,
                2 => Type::INT2,
                4 => Type::INT4,
                8 => Type::INT8,
                _ => return false,
            }
        }
    };
    ($id:ident) => {
        #[cfg(feature = "pg")]
        const _: () = {
            use std::error::Error;

            use bytes::BytesMut;
            use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};

            impl<'a> FromSql<'a> for $id {
                #[inline]
                fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
                    Ok(Self::from_bits_truncate(
                        FromSql::from_sql(ty, raw)?
                    ))
                }

                impl_pg_for_bitflags!(@ACCEPTS $id);
            }

            impl ToSql for $id {
                #[inline]
                fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
                where
                    Self: Sized,
                {
                    self.bits().to_sql(ty, out)
                }

                impl_pg_for_bitflags!(@ACCEPTS $id);
                to_sql_checked!();
            }
        };
    };
}