macro_rules! bitflags2 {
    (@DOC #[doc = $doc:literal]) => { concat!($doc, "\n") };
    (@DOC #[$meta:meta]) => {""};

    (
        $(#[$outer:meta])*
        $vis:vis struct $BitFlags:ident: $T:ty $(where $tag:literal)? {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $Flag:tt = $value:expr;
            )*
        }

        $($t:tt)*
    ) => {
        bitflags::bitflags! {
            $(#[$outer])*
            $vis struct $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    const $Flag = $value;
                )*
            }
        }

        #[cfg(feature = "ts")]
        const _: () = {
            use ts_bindgen::{Discriminator, TypeScriptDef, TypeScriptType};

            impl TypeScriptDef for $BitFlags {
                #[allow(clippy::vec_init_then_push, clippy::manual_is_power_of_two)]
                fn register(registry: &mut ts_bindgen::TypeRegistry) -> TypeScriptType {
                    // for bitflags with 16 or more bits, we decompose the bitflags
                    // values into bit positions, (1 << 20) becomes 20,
                    // at the cost of excluding combinations of flags
                    if size_of::<Self>() >= 16 {
                        let bits_name = concat!(stringify!($BitFlags), "Bit");
                        let raw_name = concat!("Raw", stringify!($BitFlags));

                        if registry.contains(bits_name) {
                            return TypeScriptType::Named(raw_name);
                        }

                        registry.add_external(raw_name);

                        eprintln!(
                            "Note: Generating TypeScript type for {} as bit positions, relying on external type {raw_name} for usage",
                            stringify!($BitFlags)
                        );

                        let mut members = Vec::new();
                        let mut combinations = Vec::new();

                        $(
                            let value = Self::$Flag.bits();

                            // if a power of 2, add to members
                            if (value & (value - 1)) == 0 {
                                members.push((
                                    stringify!($Flag).into(),
                                    Some(Discriminator::Simple(value.ilog2() as _)),
                                    concat!($(bitflags2!(@DOC #[$inner $($args)*])),*).trim().into(),
                                ));
                            } else {
                                let mut bits = Vec::new();

                                for (name, v) in Self::$Flag.iter_names() {
                                    let v = v.bits();

                                    if (v & (v - 1)) == 0 {
                                        bits.push(TypeScriptType::EnumValue(concat!(stringify!($BitFlags), "Bit"), name));
                                    }
                                }

                                if !bits.is_empty() {
                                    combinations.push((
                                        concat!(stringify!($BitFlags), "Bit_", stringify!($Flag)),
                                        TypeScriptType::ArrayLiteral(bits),
                                        concat!($(bitflags2!(@DOC #[$inner $($args)*])),*).trim(),
                                    ));
                                }
                            }
                        )*

                        registry.insert(
                            bits_name,
                            TypeScriptType::ConstEnum(members),
                            concat!("Bit positions for ", stringify!($BitFlags)),
                        );

                        // insert _after_ the bit positions enum
                        for (name, ty, doc) in combinations {
                            registry.insert(name, ty, doc);
                        }

                        // export const $BitFlagsBit_ALL = [...$BitFlagsBit.values()];
                        registry.insert(concat!(stringify!($BitFlags), "Bit_ALL"), {
                            let mut bits = Vec::new();

                            for (name, value) in Self::all().iter_names() {
                                let v = value.bits();

                                if (v & (v - 1)) == 0 {
                                    bits.push(TypeScriptType::EnumValue(concat!(stringify!($BitFlags), "Bit"), name));
                                }
                            }

                            TypeScriptType::ArrayLiteral(bits)
                        }, concat!("All bit positions of ", stringify!($BitFlags)));

                        return TypeScriptType::Named(raw_name);
                    }

                    // regular enum
                    let name = stringify!($BitFlags);
                    let ty = TypeScriptType::Named(name);

                    if registry.contains(name) {
                        return ty;
                    }

                    let mut members = Vec::new();

                    $(
                        members.push((
                            stringify!($Flag).into(),
                            Some(Discriminator::BinaryHex(Self::$Flag.bits().into())),
                            concat!($(bitflags2!(@DOC #[$inner $($args)*])),*).trim().into(),
                        ));
                    )*

                    members.push((
                        "ALL".into(),
                        Some(Discriminator::BinaryHex(Self::all().bits().into())),
                        concat!("All bitflags of ", stringify!($BitFlags)).into(),
                    ));

                    registry.insert(
                        name,
                        TypeScriptType::ConstEnum(members),
                        concat!("Bitflags for ", stringify!($BitFlags)),
                    );

                    $( registry.tag(name, $tag); )?

                    ty
                }
            }
        };

        bitflags2!($($t)*);
    };

    () => {};
}

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
            use core::error::Error;

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
            use core::error::Error;
            use num_traits::{FromPrimitive, ToPrimitive};

            use bytes::BytesMut;
            use postgres_types::{accepts, to_sql_checked, FromSql, IsNull, ToSql, Type};

            #[derive(Debug, Clone, Copy)]
            struct FromSqlError(i64);

            impl core::fmt::Display for FromSqlError {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(
                        f,
                        concat!("Unable to convert {} to type '", stringify!($id), "'"),
                        self.0
                    )
                }
            }

            impl core::error::Error for FromSqlError {}

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
                    match core::mem::size_of::<Self>() {
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
        #[cfg(feature = "schema")]
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
                    let size = core::mem::size_of::<Self>();
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
