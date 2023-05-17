#[macro_export]
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
