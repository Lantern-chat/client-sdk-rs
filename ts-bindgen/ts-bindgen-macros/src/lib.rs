#![allow(dead_code, unused)]

use proc_macro2::TokenStream;
use quote::quote;

use syn::{Data, Expr, Fields, Ident, Lit, Meta, Variant};

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
        non_const: false,
        includes: Vec::new(),
        max: false,
        tags: Vec::new(),
        rename: None,
        comment: extract_doc_comments(&input.attrs),
    };

    if let Err(e) = ctxt.check() {
        return e.into_compile_error().into();
    }

    if let Err(e) = attrs.parse_ts(&input.attrs) {
        return e.into_compile_error().into();
    }

    let name = Ident::new(
        match attrs.rename {
            Some(ref name) => name,
            None => attrs.serde.name().serialize_name(),
        },
        input.ident.span(),
    );

    let rust_name = input.ident;

    let includes = std::mem::take(&mut attrs.includes);
    let tags = std::mem::take(&mut attrs.tags);

    let inner = match input.data {
        Data::Enum(data) => derive_enum(data, name.clone(), attrs),
        Data::Struct(data) => derive_struct(data, name.clone(), attrs),
        Data::Union(_) => return compile_error_str("Unions are not supported").into(),
    };

    proc_macro::TokenStream::from(quote! {
        impl ts_bindgen::TypeScriptDef for #rust_name {
            fn register(registry: &mut ts_bindgen::TypeRegistry) -> ts_bindgen::TypeScriptType {
                if registry.contains(stringify!(#name)) {
                    return ts_bindgen::TypeScriptType::Named(stringify!(#name));
                }

                #( #includes::register(registry); )*

                #( registry.tag(stringify!(#name), #tags); )*

                #inner

                ts_bindgen::TypeScriptType::Named(stringify!(#name))
            }
        }
    })
}

struct ItemAttributes {
    serde: SerdeContainer,

    /// Item comment
    comment: String,

    /// Put interface definitions directly in unions, rather than as a named type.
    inline: bool,

    /// Prefer a regular enum for enums with explicit discriminants.
    non_const: bool,

    /// Emit a __MAX discriminator for enums with explicit discriminants.
    max: bool,

    /// Include other types in the generated register function.
    includes: Vec<Ident>,

    /// Tags to give the type in the registry
    tags: Vec<String>,

    /// Only rename the TypeScript type, not the serde type.
    rename: Option<String>,
}

impl ItemAttributes {
    pub fn parse_ts(&mut self, attrs: &[syn::Attribute]) -> syn::Result<()> {
        for attr in attrs {
            if !attr.path().is_ident("ts") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("inline") {
                    self.inline = true;
                }

                if meta.path.is_ident("max") {
                    self.max = true;
                }

                if meta.path.is_ident("non_const") {
                    self.non_const = true;
                }

                if meta.path.is_ident("include") {
                    meta.parse_nested_meta(|meta| {
                        self.includes.push(meta.path.get_ident().unwrap().clone());

                        Ok(())
                    })?;
                }

                if meta.path.is_ident("tag") {
                    let tag: syn::LitStr = meta.value()?.parse()?;

                    self.tags.push(tag.value());
                }

                if meta.path.is_ident("rename") {
                    let rename: syn::LitStr = meta.value()?.parse()?;

                    self.rename = Some(rename.value());
                }

                Ok(())
            })?;
        }

        Ok(())
    }
}

fn derive_struct(input: syn::DataStruct, name: Ident, attrs: ItemAttributes) -> TokenStream {
    let mut out = TokenStream::new();

    let struct_comment = &attrs.comment;

    // unit types are just null
    if let Fields::Unit = input.fields {
        out.extend(if attrs.inline {
            quote! { return ts_bindgen::TypeScriptType::Null; }
        } else {
            quote! { registry.insert(stringify!(#name), ts_bindgen::TypeScriptType::Null, #struct_comment); }
        });

        return out;
    }

    let ctxt = serde_derive_internals::Ctxt::new();

    // tuple structs serialize to a tuple/array
    if let Fields::Unnamed(fields) = input.fields {
        let num_fields = fields.unnamed.len();

        // only the `num_fields == 1` case is special
        if num_fields != 1 {
            out.extend(quote! {
                let mut fields = Vec::new();
            });
        }

        for (idx, field) in fields.unnamed.into_iter().enumerate() {
            let field_attrs = SerdeField::from_ast(&ctxt, idx, &field, None, attrs.serde.default());

            let field_comment = extract_doc_comments(&field.attrs);

            // NOTE: flatten/rename is not supported for tuple structs

            let field_ty = field.ty;
            let mut ty = quote! { <#field_ty as ts_bindgen::TypeScriptDef>::register(registry) };

            // allow Optional for fields that are potentially skipped
            if field_attrs.skip_serializing_if().is_some() || field_attrs.skip_serializing() {
                if field_attrs.default().is_none() && attrs.serde.default().is_none() {
                    return compile_error_str("Cannot skip serializing a field without a default value");
                }

                ty = quote! { #ty._into_optional_internal(<#field_ty as ts_bindgen::TypeScriptDef>::_IS_OPTION) };
            }

            // if there's only one field, we can just use the field directly
            out.extend(if num_fields == 1 {
                quote! { let field = (#ty, #field_comment); }
            } else {
                quote! { fields.push((#ty, #field_comment)); }
            });
        }

        out.extend(if num_fields == 1 {
            if attrs.inline {
                quote! { field }
            } else {
                quote! {
                    // special case, concat comments
                    registry.insert(stringify!(#name), field.0, {
                        let mut cmt = #struct_comment.to_owned();

                        // add a newline if there is a comment
                        if !cmt.is_empty() && !field.1.is_empty() {
                            cmt.push('\n');
                        }

                        cmt.push_str(field.1);

                        cmt
                    });
                }
            }
        } else if attrs.inline {
            quote! { return ts_bindgen::TypeScriptType::Tuple(fields); }
        } else {
            quote! {
                registry.insert(stringify!(#name), ts_bindgen::TypeScriptType::Tuple(fields), #struct_comment);
            }
        });
    } else {
        let Fields::Named(fields) = input.fields else { unreachable!() };

        out.extend(quote! {
            let mut members = Vec::new();
        });

        let mut flattened = Vec::new();

        for (idx, field) in fields.named.into_iter().enumerate() {
            let mut field_attrs = SerdeField::from_ast(&ctxt, idx, &field, None, attrs.serde.default());

            let field_comment = extract_doc_comments(&field.attrs);

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
            field_attrs.rename_by_rules(attrs.serde.rename_all_rules());
            let name = field_attrs.name().serialize_name();

            out.extend(quote! {
                members.push((#name.into(), #ty, #field_comment.into()));
            });
        }

        let num_extends = flattened.len();

        out.extend(if attrs.inline {
            quote! { return ts_bindgen::TypeScriptType::interface(members, #num_extends) #(.flatten(#flattened))*; }
        } else {
            quote! {
                let ty = ts_bindgen::TypeScriptType::interface(members, #num_extends) #(.flatten(#flattened))*;

                registry.insert(stringify!(#name), ty, #struct_comment);
            }
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

        let variant_comment = extract_doc_comments(&variant.attrs);

        (variant, variant_attrs, interface, variant_comment)
    }));

    out.extend(quote! {
        // union of variants
        let mut variants = Vec::new();
    });

    let enum_comment = &attrs.comment;

    if actual_enum {
        if let Err(e) = ctxt.check() {
            return e.into_compile_error();
        }

        if num_discriminants == 0 {
            // enum with no fields or discriminants, so it should be a regular enum with string values
            for (variant, variant_attrs, _, variant_comment) in variants {
                let variant_ident = variant.ident;
                let variant_name = variant_attrs.name().serialize_name();

                out.extend(quote! {
                    variants.push((
                        stringify!(#variant_ident).into(),
                        Some(ts_bindgen::Discriminator::String(#variant_name)),
                        #variant_comment.into(),
                    ));
                });
            }

            // use a real enum for enums with string values
            out.extend(quote! { let ty = ts_bindgen::TypeScriptType::Enum(variants); });
        } else {
            // explicit discriminants, so it should be a const enum
            for (variant, _, _, variant_comment) in variants {
                let name = variant.ident;

                let discriminant = match variant.discriminant {
                    Some((_, Expr::Lit(lit))) => quote! { Some(ts_bindgen::Discriminator::Simple(#lit)) },
                    _ => quote! { None },
                };

                out.extend(quote! {
                    variants.push((
                        stringify!(#name).into(),
                        #discriminant,
                        #variant_comment.into(),
                    ));
                });
            }

            if attrs.max {
                out.extend(quote! {
                    variants.push((
                        "__MAX".into(),
                        None,
                        "Max value for the enum".into(),
                    ));
                });
            }

            let ty = if attrs.non_const {
                quote! { Enum }
            } else {
                quote! { ConstEnum }
            };

            out.extend(quote! { let ty = ts_bindgen::TypeScriptType::#ty(variants); });
        }

        out.extend(quote! { registry.insert(stringify!(#name), ty, #enum_comment); });

        return out;
    }

    for (variant, variant_attrs, interface_name, variant_comment) in variants {
        let variant_name = variant_attrs.name().serialize_name();

        match variant.fields {
            // unit fields are just equal to null
            Fields::Unit => out.extend(match attrs.serde.tag() {
                // { "variant_a": null } | { "variant_b": null } | ...
                TagType::External => quote! {
                    let mut members = Vec::new();
                    members.push((#variant_name.into(), ts_bindgen::TypeScriptType::Null));

                    variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                },

                // { tag: "variant_a" } | { tag: "variant_b" } | ...
                TagType::Internal { tag } => quote! {
                    let mut members = Vec::new();
                    members.push((
                        #tag.into(),
                        ts_bindgen::TypeScriptType::string_value(#variant_name),
                        #variant_comment.into(),
                    ));

                    variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                },

                // { tag: "variant_a", content: null } | { tag: "variant_b", content: null } | ...
                TagType::Adjacent { tag, content } => quote! {
                    let mut members = Vec::new();
                    members.push((#tag.into(), ts_bindgen::TypeScriptType::string_value(#variant_name), #variant_comment.into()));
                    members.push((#content.into(), ts_bindgen::TypeScriptType::Null, "".into()));

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

                    let field_comment = extract_doc_comments(&field.attrs);

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
                        members.push((#name.into(), #ty, #field_comment.into()));
                    });
                }

                let num_extends = flattened.len();

                // TODO: Handle inline interfaces
                out.extend(match attrs.serde.tag() {
                    // { "variant_a": variant_a } | { "variant_b": variant_b } | ...
                    TagType::External => quote! {
                        let ty = ts_bindgen::TypeScriptType::interface(members, #num_extends) #(.flatten(#flattened))*;
                        registry.insert(#interface_name, ty, #variant_comment);

                        let mut members = Vec::new();
                        members.push((#variant_name.into(), ts_bindgen::TypeScriptType::Named(#interface_name), #variant_comment.into()));

                        variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                    },

                    // { tag: "variant_a", ...variant_a } | { tag: "variant_b", ...variant_b } | ...
                    TagType::Internal { tag } => quote! {
                        members.push((#tag.into(), ts_bindgen::TypeScriptType::string_value(#variant_name), #variant_comment.into()));
                    },

                    // { tag: "variant_a", content: variant_a } |
                    // { tag: "variant_b", content: variant_b } | ...
                    TagType::Adjacent { tag, content } => quote! {
                        let ty = ts_bindgen::TypeScriptType::interface(members, #num_extends) #(.flatten(#flattened))*;
                        registry.insert(#interface_name, ty, #variant_comment);

                        let mut members = Vec::new();
                        members.push((#tag.into(), ts_bindgen::TypeScriptType::string_value(#variant_name), "".into()));
                        members.push((#content.into(), ts_bindgen::TypeScriptType::Named(#interface_name), "".into()));

                        variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                    },

                    // variant_a | variant_b | ...
                    TagType::None => quote! {
                        let ty = ts_bindgen::TypeScriptType::interface(members, #num_extends) #(.flatten(#flattened))*;
                        registry.insert(#interface_name, ty, #variant_comment);

                        variants.push(ts_bindgen::TypeScriptType::Named(#interface_name));
                    },
                });
            }
            // tuple fields are just equal to an array
            Fields::Unnamed(fields_unnamed) => {
                let num_fields = fields_unnamed.unnamed.len();

                // only the `num_fields == 1` case is special
                if num_fields != 1 {
                    out.extend(quote! {
                        let mut fields = Vec::new();
                    });
                }

                for (idx, field) in fields_unnamed.unnamed.into_iter().enumerate() {
                    let field_attrs = SerdeField::from_ast(&ctxt, idx, &field, None, attrs.serde.default());

                    let field_comment = extract_doc_comments(&field.attrs);

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

                    // if there's only one field, we can just use the field directly
                    out.extend(if num_fields == 1 {
                        quote! { let field = (#ty, #field_comment); }
                    } else {
                        quote! { fields.push((#ty, #field_comment)); }
                    });
                }

                // unwrap single field tuple variants
                if num_fields == 1 {
                    // one variant field is just equal to the field
                    out.extend(match attrs.serde.tag() {
                        // { "variant_a": variant_a } | { "variant_b": variant_b } | ...
                        TagType::External => quote! {
                            let mut members = Vec::new();
                            members.push((#variant_name.into(), field.0, field.1.into()));

                            variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                        },

                        // { tag: "variant_a", ...variant_a } | { tag: "variant_b", ...variant_b } | ...
                        TagType::Internal { tag } => quote! {
                            let mut members = Vec::new();
                            members.push((#tag.into(), ts_bindgen::TypeScriptType::string_value(#variant_name), field.1.into()));

                            // NOTE: This will fail at runtime if the field is not a composite type
                            variants.push(ts_bindgen::TypeScriptType::interface(members, 1).flatten(field.0));
                        },

                        // { tag: "variant_a", content: variant_a } |
                        // { tag: "variant_b", content: variant_b } | ...
                        TagType::Adjacent { tag, content } => quote! {
                            let mut members = Vec::new();
                            members.push((#tag.into(), ts_bindgen::TypeScriptType::string_value(#variant_name), "".into()));
                            members.push((#content.into(), field.0, field.1.into()));

                            variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                        },

                        // variant_a | variant_b | ...
                        TagType::None => quote! { variants.push(field.0); },
                    });

                    continue;
                }

                // TODO: Handle inline enums
                out.extend(match attrs.serde.tag() {
                    // { "variant_a": variant_a } | { "variant_b": variant_b } | ...
                    TagType::External => quote! {
                        registry.insert(#interface_name, ts_bindgen::TypeScriptType::Tuple(fields));

                        let mut members = Vec::new();
                        members.push((#variant_name.into(), ts_bindgen::TypeScriptType::Named(#interface_name), #variant_comment.into()));

                        variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                    },

                    // { tag: "variant_a", ...variant_a } | { tag: "variant_b", ...variant_b } | ...
                    TagType::Internal { tag } => quote! {
                        compile_error!("Internal tags are not supported for tuple variants");
                    },

                    // { tag: "variant_a", content: variant_a } |
                    // { tag: "variant_b", content: variant_b } | ...
                    TagType::Adjacent { tag, content } => quote! {
                        registry.insert(#interface_name, ts_bindgen::TypeScriptType::Tuple(fields), #variant_comment);

                        let mut members = Vec::new();
                        members.push((#tag.into(), ts_bindgen::TypeScriptType::string_value(#variant_name), "".into()));
                        members.push((#content.into(), ts_bindgen::TypeScriptType::Named(#interface_name), "".into()));

                        variants.push(ts_bindgen::TypeScriptType::interface(members, 0));
                    },

                    // variant_a | variant_b | ...
                    TagType::None => quote! {
                        registry.insert(#interface_name, ts_bindgen::TypeScriptType::Tuple(fields), #variant_comment);

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
        registry.insert(stringify!(#name), ts_bindgen::TypeScriptType::Union(variants), #enum_comment);
    });

    out
}

fn compile_error_str(msg: &str) -> TokenStream {
    quote! { ::core::compile_error!(#msg) }
}

fn extract_doc_comments(attrs: &[syn::Attribute]) -> String {
    let mut comment = String::new();

    for attr in attrs {
        let Meta::NameValue(ref nv) = attr.meta else { continue };

        if nv.path.is_ident("doc") {
            if let Expr::Lit(syn::ExprLit {
                lit: Lit::Str(ref lit), ..
            }) = nv.value
            {
                if !comment.is_empty() {
                    comment.push('\n');
                }
                comment.push_str(&lit.value());
            }
        }
    }

    comment
}
