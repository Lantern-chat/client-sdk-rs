#![allow(dead_code, unused)]

use proc_macro2::TokenStream;
use quote::quote;

use syn::{Data, Expr, Fields, Ident, Meta, Variant};

use serde_derive_internals::attr::{
    Container as SerdeContainer, Field as SerdeField, RenameAllRules, RenameRule, TagType, Variant as SerdeVariant,
};

#[proc_macro_derive(TypeScriptDef, attributes(ts, serde))]
pub fn derive_typescript_def(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let ctxt = serde_derive_internals::Ctxt::new();

    let mut attrs = ItemAttributes {
        serde: SerdeContainer::from_ast(&ctxt, &input),
        inline: false,
    };

    if let Err(e) = ctxt.check() {
        return e.into_compile_error().into();
    }

    attrs.parse_ts(&input.attrs);

    let name = input.ident;

    let inner = match input.data {
        Data::Enum(data) => derive_enum(data, name.clone(), attrs),
        Data::Struct(data) => derive_struct(data, name.clone(), attrs),
        Data::Union(_) => unimplemented!(),
    };

    proc_macro::TokenStream::from(quote! {
        impl ts_bindgen::TypeScriptDef for #name {
            fn register(registry: &mut ts_bindgen::TypeRegistry) -> ts_bindgen::TypeScriptType {
                if registry.contains(stringify!(#name)) {
                    return ts_bindgen::TypeScriptType::Named(stringify!(#name));
                }

                #inner
            }
        }
    })
}

struct ItemAttributes {
    serde: SerdeContainer,

    /// Put interface definitions directly in unions, rather than as a named type.
    inline: bool,
}

impl ItemAttributes {
    pub fn parse_ts(&mut self, attrs: &[syn::Attribute]) {
        for attr in attrs {
            if attr.path().is_ident("ts") {
                // TODO: parse ts attributes
                continue;
            }
        }
    }
}

fn derive_struct(input: syn::DataStruct, name: Ident, attrs: ItemAttributes) -> TokenStream {
    let mut out = TokenStream::new();

    // unit types are just null
    if let Fields::Unit = input.fields {
        out.extend(if attrs.inline {
            quote! { ts_bindgen::TypeScriptType::Null }
        } else {
            quote! {
                registry.insert(stringify!(#name), ts_bindgen::TypeScriptType::Null);
                ts_bindgen::TypeScriptType::Named(stringify!(#name))
            }
        });

        return out;
    }

    let ctxt = serde_derive_internals::Ctxt::new();

    // tuple structs serialize to a tuple/array
    if let Fields::Unnamed(fields) = input.fields {
        out.extend(quote! {
            let mut fields = Vec::new();
        });

        let num_fields = fields.unnamed.len();

        for (idx, field) in fields.unnamed.into_iter().enumerate() {
            let field_attrs = SerdeField::from_ast(&ctxt, idx, &field, None, attrs.serde.default());

            // NOTE: flatten/rename is not supported for tuple structs

            let field_ty = field.ty;
            let mut ty = quote! { <#field_ty as ts_bindgen::TypeScriptDef>::register(registry) };

            // allow Null for optional fields
            // if !field_attrs.default().is_none() || !attrs.serde.default().is_none() {
            //     ty = quote! { #ty.into_nullable() };
            // }

            // allow Optional for fields that are potentially skipped
            if field_attrs.skip_serializing_if().is_some() || field_attrs.skip_serializing() {
                if field_attrs.default().is_none() && attrs.serde.default().is_none() {
                    return compile_error_str("Cannot skip serializing a field without a default value");
                }

                ty = quote! { #ty._into_optional_internal(<#field_ty as ts_bindgen::TypeScriptDef>::_IS_OPTION) };
            }

            out.extend(quote! { fields.push(#ty); });
        }

        if num_fields == 1 {
            out.extend(quote! {
                let ty = fields.pop().unwrap();
            });

            out.extend(if attrs.inline {
                quote! { ty }
            } else {
                quote! {
                    registry.insert(stringify!(#name), ty);
                    ts_bindgen::TypeScriptType::Named(stringify!(#name))
                }
            });
        } else {
            out.extend(quote! {
                registry.insert(stringify!(#name), ts_bindgen::TypeScriptType::Tuple(fields));
                ts_bindgen::TypeScriptType::Named(stringify!(#name))
            });
        }
    } else {
        let Fields::Named(fields) = input.fields else { unreachable!() };

        out.extend(quote! {
            let mut members = Vec::new();
        });

        let mut flattened = Vec::new();

        for (idx, field) in fields.named.into_iter().enumerate() {
            let mut field_attrs = SerdeField::from_ast(&ctxt, idx, &field, None, attrs.serde.default());

            let field_ty = field.ty;
            let mut ty = quote! { <#field_ty as ts_bindgen::TypeScriptDef>::register(registry) };

            // allow Null for optional fields
            // if !field_attrs.default().is_none() || !attrs.serde.default().is_none() {
            //     ty = quote! { #ty.into_nullable() };
            // }

            // allow Optional for fields that are potentially skipped
            if field_attrs.skip_serializing_if().is_some() || field_attrs.skip_serializing() {
                if field_attrs.default().is_none() && attrs.serde.default().is_none() {
                    ctxt.error_spanned_by(field.ident, "Cannot skip serializing a field without a default value");
                    break;
                }

                ty = quote! { #ty._into_optional_internal(<#field_ty as ts_bindgen::TypeScriptDef>::_IS_OPTION) };
            }

            if field_attrs.flatten() {
                flattened.push(ty);
                continue; // name is not needed
            }

            // apply any renaming rules
            field_attrs.rename_by_rules(attrs.serde.rename_all_rules());
            let name = field_attrs.name().serialize_name();

            out.extend(quote! {
                members.push((#name.to_owned(), #ty));
            });
        }

        let num_extends = flattened.len();

        out.extend(quote! {
            let ty = ts_bindgen::TypeScriptType::interface(members, #num_extends) #(.flatten(#flattened))*;

            registry.insert(stringify!(#name), ty);

            ts_bindgen::TypeScriptType::Named(stringify!(#name))
        });
    }

    if let Err(e) = ctxt.check() {
        return e.into_compile_error();
    }

    out
}

fn derive_enum(input: syn::DataEnum, name: Ident, attrs: ItemAttributes) -> TokenStream {
    let mut out = TokenStream::new();

    let ctxt = serde_derive_internals::Ctxt::new();

    let mut actual_enum = true;
    let mut num_discriminants = 0;

    let variants = Vec::from_iter(input.variants.into_iter().map(|variant| {
        let mut variant_attrs = SerdeVariant::from_ast(&ctxt, &variant);

        variant_attrs.rename_by_rules(attrs.serde.rename_all_rules());

        // when an interface is generated for a variant, use this name
        let interface = format!("{}{}", name, variant.ident);

        actual_enum &= matches!(variant.fields, Fields::Unit);

        num_discriminants += variant.discriminant.is_some() as usize;

        (variant, variant_attrs, interface)
    }));

    out.extend(quote! {
        // union of variants
        let mut variants = Vec::new();
    });

    if actual_enum {
        if let Err(e) = ctxt.check() {
            return e.into_compile_error();
        }

        if num_discriminants == 0 {
            for (variant, variant_attrs, ..) in variants {
                let variant_ident = variant.ident;
                let variant_name = variant_attrs.name().serialize_name();

                out.extend(quote! {
                    variants.push((stringify!(#variant_ident).to_owned(), Some(ts_bindgen::Discriminator::String(#variant_name))));
                });
            }
        } else {
            for (variant, ..) in variants {
                let name = variant.ident;

                let discriminant = match variant.discriminant {
                    Some((_, Expr::Lit(lit))) => quote! { Some(ts_bindgen::Discriminator::Simple(#lit)) },
                    _ => quote! { None },
                };

                out.extend(quote! {
                    variants.push((stringify!(#name).to_owned(), #discriminant));
                });
            }
        }

        out.extend(quote! {
            let ty = ts_bindgen::TypeScriptType::ConstEnum(variants);
            registry.insert(stringify!(#name), ty);

            ts_bindgen::TypeScriptType::Named(stringify!(#name))
        });

        return out;
    }

    for (variant, variant_attrs, interface_name) in variants {
        let variant_name = variant_attrs.name().serialize_name();

        match variant.fields {
            // unit fields are just equal to null
            Fields::Unit => out.extend(match attrs.serde.tag() {
                // { "variant_a": null } | { "variant_b": null } | ...
                TagType::External => quote! {
                    let mut members = Vec::new();
                    members.push((#variant_name.to_owned(), ts_bindgen::TypeScriptType::Null));

                    variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                },

                // { tag: "variant_a" } | { tag: "variant_b" } | ...
                TagType::Internal { tag } => quote! {
                    let mut members = Vec::new();
                    members.push((#tag.to_owned(), ts_bindgen::TypeScriptType::string_value(#variant_name)));

                    variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                },

                // { tag: "variant_a", content: null } | { tag: "variant_b", content: null } | ...
                TagType::Adjacent { tag, content } => quote! {
                    let mut members = Vec::new();
                    members.push((#tag.to_owned(), ts_bindgen::TypeScriptType::string_value(#variant_name)));
                    members.push((#content.to_owned(), ts_bindgen::TypeScriptType::Null));

                    variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                },

                // variant_a | variant_b | ...
                TagType::None => quote! { variants.push(ts_bindgen::TypeScriptType::Null); },
            }),
            // Define a new interface for the variant
            Fields::Named(fields) => {
                let mut flattened = Vec::new();

                out.extend(quote! {
                    let mut members = Vec::new();
                });

                for (idx, field) in fields.named.into_iter().enumerate() {
                    let mut field_attrs = SerdeField::from_ast(&ctxt, idx, &field, None, attrs.serde.default());

                    let field_ty = field.ty;
                    let mut ty = quote! { <#field_ty as ts_bindgen::TypeScriptDef>::register(registry) };

                    // allow Optional for fields that are potentially skipped
                    if field_attrs.skip_serializing_if().is_some() || field_attrs.skip_serializing() {
                        if field_attrs.default().is_none() && attrs.serde.default().is_none() {
                            ctxt.error_spanned_by(field.ident, "Cannot skip serializing a field without a default value");
                            break;
                        }

                        ty = quote! { #ty._into_optional_internal(<#field_ty as ts_bindgen::TypeScriptDef>::_IS_OPTION) };
                    }

                    if field_attrs.flatten() {
                        flattened.push(ty);
                        continue; // name is not needed
                    }

                    // apply any renaming rules
                    field_attrs.rename_by_rules(variant_attrs.rename_all_rules());
                    let name = field_attrs.name().serialize_name();

                    out.extend(quote! {
                        members.push((#name.to_owned(), #ty));
                    });
                }

                let num_extends = flattened.len();

                // TODO: Handle inline interfaces
                out.extend(match attrs.serde.tag() {
                    // { "variant_a": variant_a } | { "variant_b": variant_b } | ...
                    TagType::External => quote! {
                        let ty = ts_bindgen::TypeScriptType::interface(members, #num_extends) #(.flatten(#flattened))*;
                        registry.insert(#interface_name, ty);

                        let mut members = Vec::new();
                        members.push((#variant_name.to_owned(), ts_bindgen::TypeScriptType::Named(#interface_name)));

                        variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                    },

                    // { tag: "variant_a", ...variant_a } | { tag: "variant_b", ...variant_b } | ...
                    TagType::Internal { tag } => quote! {
                        members.push((#tag.to_owned(), ts_bindgen::TypeScriptType::string_value(#variant_name)));
                    },

                    // { tag: "variant_a", content: variant_a } |
                    // { tag: "variant_b", content: variant_b } | ...
                    TagType::Adjacent { tag, content } => quote! {
                        let ty = ts_bindgen::TypeScriptType::interface(members, #num_extends) #(.flatten(#flattened))*;
                        registry.insert(#interface_name, ty);

                        let mut members = Vec::new();
                        members.push((#tag.to_owned(), ts_bindgen::TypeScriptType::string_value(#variant_name)));
                        members.push((#content.to_owned(), ts_bindgen::TypeScriptType::Named(#interface_name)));

                        variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                    },

                    // variant_a | variant_b | ...
                    TagType::None => quote! {
                        let ty = ts_bindgen::TypeScriptType::interface(members, #num_extends) #(.flatten(#flattened))*;
                        registry.insert(#interface_name, ty);

                        variants.push(ts_bindgen::TypeScriptType::Named(#interface_name));
                    },
                });
            }
            // tuple fields are just equal to an array
            Fields::Unnamed(fields_unnamed) => {
                out.extend(quote! {
                    let mut fields = Vec::new();
                });

                let num_fields = fields_unnamed.unnamed.len();

                for (idx, field) in fields_unnamed.unnamed.into_iter().enumerate() {
                    let field_attrs = SerdeField::from_ast(&ctxt, idx, &field, None, attrs.serde.default());

                    let field_ty = field.ty;
                    let mut ty = quote! { <#field_ty as ts_bindgen::TypeScriptDef>::register(registry) };

                    // allow Optional for fields that are potentially skipped
                    if field_attrs.skip_serializing_if().is_some() || field_attrs.skip_serializing() {
                        if field_attrs.default().is_none() && attrs.serde.default().is_none() {
                            ctxt.error_spanned_by(field.ident, "Cannot skip serializing a field without a default value");
                            break;
                        }

                        ty = quote! { #ty._into_optional_internal(<#field_ty as ts_bindgen::TypeScriptDef>::_IS_OPTION) };
                    }

                    out.extend(quote! { fields.push(#ty); });
                }

                // unwrap single field tuple variants
                if num_fields == 1 {
                    out.extend(quote! {
                        let field = fields.pop().unwrap();
                    });

                    // one variant field is just equal to the field
                    out.extend(match attrs.serde.tag() {
                        // { "variant_a": variant_a } | { "variant_b": variant_b } | ...
                        TagType::External => quote! {
                            let mut members = Vec::new();
                            members.push((#variant_name.to_owned(), field));

                            variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                        },

                        // { tag: "variant_a", ...variant_a } | { tag: "variant_b", ...variant_b } | ...
                        TagType::Internal { tag } => quote! {
                            let mut members = Vec::new();
                            members.push((#tag.to_owned(), ts_bindgen::TypeScriptType::string_value(#variant_name)));

                            // NOTE: This will fail at runtime if the field is not a composite type
                            variants.push(ts_bindgen::TypeScriptType::interface(members, 1).flatten(field));
                        },

                        // { tag: "variant_a", content: variant_a } |
                        // { tag: "variant_b", content: variant_b } | ...
                        TagType::Adjacent { tag, content } => quote! {
                            let mut members = Vec::new();
                            members.push((#tag.to_owned(), ts_bindgen::TypeScriptType::string_value(#variant_name)));
                            members.push((#content.to_owned(), field));

                            variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                        },

                        // variant_a | variant_b | ...
                        TagType::None => quote! { variants.push(field); },
                    });

                    continue;
                }

                // TODO: Handle inline enums
                out.extend(match attrs.serde.tag() {
                    // { "variant_a": variant_a } | { "variant_b": variant_b } | ...
                    TagType::External => quote! {
                        registry.insert(#interface_name, ts_bindgen::TypeScriptType::Tuple(fields));

                        let mut members = Vec::new();
                        members.push((#variant_name.to_owned(), ts_bindgen::TypeScriptType::Named(#interface_name)));

                        variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                    },

                    // { tag: "variant_a", ...variant_a } | { tag: "variant_b", ...variant_b } | ...
                    TagType::Internal { tag } => quote! {
                        compile_error!("Internal tags are not supported for tuple variants");
                    },

                    // { tag: "variant_a", content: variant_a } |
                    // { tag: "variant_b", content: variant_b } | ...
                    TagType::Adjacent { tag, content } => quote! {
                        registry.insert(#interface_name, ts_bindgen::TypeScriptType::Tuple(fields));

                        let mut members = Vec::new();
                        members.push((#tag.to_owned(), ts_bindgen::TypeScriptType::string_value(#variant_name)));
                        members.push((#content.to_owned(), ts_bindgen::TypeScriptType::Named(#interface_name)));

                        variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                    },

                    // variant_a | variant_b | ...
                    TagType::None => quote! {
                        registry.insert(#interface_name, ts_bindgen::TypeScriptType::Tuple(fields));

                        variants.push(ts_bindgen::TypeScriptType::Named(#interface_name));
                    },
                });
            }
        }
    }

    if let Err(e) = ctxt.check() {
        return e.into_compile_error();
    }

    out.extend(quote! {
        let ty = ts_bindgen::TypeScriptType::Union(variants);
        registry.insert(stringify!(#name), ty);

        ts_bindgen::TypeScriptType::Named(stringify!(#name))
    });

    out
}

fn compile_error_str(msg: &str) -> TokenStream {
    quote! { ::core::compile_error!(#msg) }
}
