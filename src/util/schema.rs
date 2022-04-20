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
                    let mut obj = SchemaObject {
                        metadata: Some(Box::new(Metadata {
                            description: Some(format!("{} Bitflags", stringify!($name))),
                            examples: vec![json!($name::all().bits())],
                            ..Default::default()
                        })),
                        instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Number))),
                        ..Default::default()
                    };

                    obj.number().maximum = Some($name::all().bits() as _);

                    Schema::Object(obj)
                }
            }
        };
    };
}
