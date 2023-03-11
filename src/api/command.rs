use std::fmt;

use http::{HeaderMap, Method};

pub(crate) mod sealed {
    pub trait Sealed {}
}

use crate::models::Permission;

bitflags::bitflags! {
    pub struct CommandFlags: u8 {
        const AUTHORIZED    = 1 << 0;
        const HAS_BODY      = 1 << 1;
    }
}

/// Client Command, tells the client to perform specific requests
///
/// A "Command" is a mid-level abstraction around REST endpoints and their bodies. Not perfect,
/// but zero-cost and simple. Other abstractions can be built on top of it.
///
/// A command consists of three parts: the URL, the "body", and headers.
///
/// For the case of `GET`/`OPTIONS` commands, the body becomes query parameters.
pub trait Command: sealed::Sealed {
    /// Object returned from the server as the result of a command
    type Result: serde::de::DeserializeOwned;
    type Body: serde::Serialize;

    /// HTTP Method used to execute the command
    const METHOD: Method;

    const FLAGS: CommandFlags;

    /// Serialize/format the REST path (without query)
    fn format_path<W: fmt::Write>(&self, w: W) -> fmt::Result;

    fn body(&self) -> &Self::Body;

    /// Hint given to preallocate body size, only used for query strings
    #[inline(always)]
    fn body_size_hint(&self) -> usize {
        0
    }

    /// Computes required permissions
    fn perms(&self) -> Permission;

    /// Insert any additional headers required to perform this command
    #[inline(always)]
    fn add_headers(&self, _map: &mut HeaderMap) {}

    #[cfg(feature = "schema")]
    fn schema(gen: &mut schemars::gen::SchemaGenerator) -> (String, okapi::openapi3::PathItem);
}

// Takes an expression like:
//  "a" / value / "b" / value2
// and converts it into a sequence of `Write` writes
macro_rules! format_path {
    ($w:expr, $this:expr, [$($value:literal),+] [/ $next:literal $(/ $tail:tt)*]) => {
        format_path!($w, $this, [$($value,)+ $next] [$(/ $tail)*]);
    };

    ($w:expr, $this:expr, [$($value:literal),+] [/ $next:tt $(/ $tail:tt)*]) => {
        $w.write_str(concat!($("/", $value),+))?;
        format_path!($w, $this, [$next] [$(/ $tail)*]);
    };

    ($w:expr, $this:expr, [$value:ident] [/ $next:tt $(/ $tail:tt)*]) => {
        write!($w, "/{}", $this.$value)?;
        format_path!($w, $this, [$next] [$(/ $tail)*]);
    };

    ($w:expr, $this:expr, [$($value:literal),*] []) => { $w.write_str(concat!($("/", $value),*))?; };
    ($w:expr, $this:expr, [$value:ident] []) => { write!($w, "/{}", $this.$value)?; };
}

// Similar to the above, but concatenates the path together for usage in schemas
macro_rules! schema_path {
    ([$($value:literal),+] [/ $next:literal $(/ $tail:tt)*]) => {
        schema_path!([$($value,)+ $next] [$(/ $tail)*])
    };

    ([$($value:literal),+] [/ $next:tt $(/ $tail:tt)*]) => {
        concat!($("/", $value,)+ schema_path!([$next] [$(/ $tail)*]))
    };

    ([$value:ident] [/ $next:tt $(/ $tail:tt)*]) => {
        concat!("/{", stringify!($value), "}", schema_path!([$next] [$(/ $tail)*]))
    };

    ([$($value:literal),*] []) => { concat!($("/", $value),*) };
    ([$value:ident] []) => { concat!("/{", stringify!($value), "}") };
}

// Macro to autogenerate most Command trait implementations.
macro_rules! command {
    (@STRUCT struct) => {};

    (@BODY_TY $name:ident) => { $name };
    (@BODY_TY) => { () };

    (@BODY_RETURN $name:ident $ret:expr) => { $ret };
    (@BODY_RETURN ) => { &() };

    // get doc comments as string literals
    (@DOC #[doc = $doc:literal]) => { concat!($doc, "\n") };
    (@DOC #[$meta:meta]) => {""};

    (@DEPRECATED #[deprecated $($any:tt)*]) => { true };
    (@DEPRECATED #[$meta:meta]) => { false };

    // only insert block if GET-ish method (i.e. body is treated as query)
    (@GET GET $c:block) => {$c};
    (@GET OPTIONS $c:block) => {$c};
    (@GET HEAD $c:block) => {$c};
    (@GET CONNECT $c:block) => {$c};
    (@GET TRACE $c:block) => {$c};
    (@GET $other:ident $c:block) => {};

    // entry point
    ($(
        // meta
        $(#[$($meta:tt)*])*

        // two symbols to differentiate auth and noauth commands (keyword struct verified in @STRUCT)
        $(+$auth_struct:ident)? $(-$noauth_struct:ident)?

        // name, result and HTTP method
        $name:ident -> $result:ty: $method:ident(
            $head:tt $(/ $tail:tt)*
        )
        // permissions
        $(where $($kind:ident::$perm:ident)|+)?

        // HTTP Headers
        $($($(#[$header_meta:meta])* $header_name:literal => $header_vis:vis $header_field:ident: $header_ty:ty),+ $(,)*)?

        // fields
        {
            $(
                $(#[$($field_meta:tt)*])*
                $field_vis:vis $field_name:ident: $field_ty:ty $(
                    // conditional additional permissions
                    where $($field_kind:ident::$field_perm:ident)|+ if $cond:expr
                )?

            ),* $(,)*

            $(
                ; // need to terminate the previous expressions

                // separate body struct that will be flattened
                $(#[$body_meta:meta])*
                struct $body_name:ident {
                    $(

                        $(#[$($body_field_meta:tt)*])*
                        $body_field_vis:vis $body_field_name:ident: $body_field_ty:ty $(
                            where $($body_field_kind:ident::$body_field_perm:ident)|+ if $body_field_cond:expr
                        )?

                    ),* $(,)*
                }
            )?
        }
    )*) => {$(
        // verify presence of exactly one `struct` without prefix
        command!(@STRUCT $($auth_struct)? $($noauth_struct)?);

        impl $crate::api::command::sealed::Sealed for $name {}
        impl $crate::api::command::Command for $name {
            type Result = $result;

            const METHOD: http::Method = http::Method::$method;

            const FLAGS: CommandFlags = CommandFlags::empty()
                $(.union((stringify!($body_name), CommandFlags::HAS_BODY).1))?
                $(.union((stringify!($auth_struct), CommandFlags::AUTHORIZED).1))?
            ;

            #[allow(unused_mut, unused_variables, deprecated)]
            fn perms(&self) -> Permission {
                let mut base = crate::perms!($($($kind::$perm)|+)?);

                let $name {
                    $(ref $field_name,)*

                    $( $(ref $header_field,)* )?

                    $(
                        body: $body_name { $(ref $body_field_name),* }
                    )?
                } = self;

                $($(
                    if $cond {
                        base |= crate::perms!($($field_kind::$field_perm)|+)
                    }
                )?)*

                base
            }

            #[inline]
            #[allow(deprecated)]
            fn format_path<W: std::fmt::Write>(&self, mut w: W) -> std::fmt::Result {
                format_path!(w, self, [$head] [$(/ $tail)*]);

                Ok(())
            }

            type Body = command!(@BODY_TY $($body_name)?);

            fn body(&self) -> &Self::Body {
                command!(@BODY_RETURN $($body_name &self.body)?)
            }

            $(
                #[inline]
                fn body_size_hint(&self) -> usize {
                    // ?value= &another=
                    0 $(+ 3 + stringify!($body_field_name).len())*
                }
            )?

            $(
                #[inline(always)]
                fn add_headers(&self, map: &mut http::HeaderMap) {
                    $(
                        map.insert($header_name, http::HeaderValue::from_maybe_shared(self.$header_field.to_string()).unwrap());
                    )+
                }
            )?

            #[cfg(feature = "schema")]
            fn schema(gen: &mut schemars::gen::SchemaGenerator) -> (String, okapi::openapi3::PathItem) {
                #![allow(unused)]

                use http::Method;
                use schemars::{JsonSchema, schema::SchemaObject, gen::SchemaGenerator};
                use okapi::openapi3::{Operation, PathItem, Parameter, ParameterValue, RefOr, Object};

                let mut path_item = PathItem::default();

                paste::paste!(path_item.[<$method:lower>]) = Some({
                    let mut op = Operation {
                        description: {
                            let description = concat!($(command!(@DOC #[$($meta)*])),*).trim();
                            if description.is_empty() { None } else { Some(description.to_owned()) }
                        },
                        ..Default::default()
                    };

                    // if has body and GET-ish
                    $(
                        command!(@GET $method {$(
                            op.parameters.push(RefOr::Object(Parameter {
                                name: stringify!($body_field_name).to_owned(),
                                location: "query".to_owned(),
                                description: {
                                    let description = concat!($(command!(@DOC #[$($body_field_meta)*])),*).trim();
                                    if description.is_empty() { None } else { Some(description.to_owned()) }
                                },
                                // TODO: Figure out a better way to detect `Option<T>` types?
                                required: !<$body_field_ty as JsonSchema>::_schemars_private_is_option(),
                                deprecated: false $(|| command!(@DEPRECATED #[$($body_field_meta)*]))*,
                                allow_empty_value: false,
                                extensions: Default::default(),
                                value: ParameterValue::Schema {
                                    style: None,
                                    explode: None,
                                    allow_reserved: false,
                                    schema: <$body_field_ty as JsonSchema>::json_schema(gen).into_object(),
                                    example: None,
                                    examples: None,
                                }
                            }));
                        )*});
                    )?

                    let response_schema_name = <$result as JsonSchema>::schema_name();

                    // if not ()
                    if response_schema_name != "Null" {
                        // TODO: Figure out how to insert and reference schema?
                        //let defs = gen.definitions_mut();
                        //op.responses.default = Some()
                    }

                    op
                });

                path_item.parameters = vec![$({
                    RefOr::Object(Parameter {
                        name: stringify!($field_name).to_owned(),
                        location: "path".to_owned(),
                        description: {
                            let description = concat!($(command!(@DOC #[$($field_meta)*])),*).trim();
                            if description.is_empty() { None } else { Some(description.to_owned()) }
                        },
                        required: true,
                        deprecated: false $(|| command!(@DEPRECATED #[$($field_meta)*]))*,
                        allow_empty_value: false,
                        extensions: Default::default(),
                        value: ParameterValue::Schema {
                            style: None,
                            explode: None,
                            allow_reserved: false,
                            schema: <$field_ty as JsonSchema>::json_schema(gen).into_object(),
                            example: None,
                            examples: None,
                        }
                    })
                },)*];

                (schema_path!([$head] [$(/ $tail)*]).to_owned(), path_item)
            }
        }

        $(#[$($meta)*])*
        #[derive(Debug)]
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        pub struct $name {
            $($(#[$($field_meta)*])* $field_vis $field_name: $field_ty, )*

            $( $($(#[$header_meta])* $header_vis $header_field: $header_ty, )* )?

            $(
                /// Body to be serialized as request body or query parameters (if GET)
                pub body: $body_name,
            )?
        }

        $(
            $(#[$body_meta])*
            #[derive(Debug, Serialize, Deserialize)]
            #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
            pub struct $body_name {
                $( $(#[$($body_field_meta)*])* $body_field_vis $body_field_name: $body_field_ty ),*
            }

            impl std::ops::Deref for $name {
                type Target = $body_name;

                #[inline]
                fn deref(&self) -> &Self::Target {
                    &self.body
                }
            }

            impl std::ops::DerefMut for $name {
                #[inline]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.body
                }
            }
        )?

        impl $name {
            #[doc = "Construct new instance from individual fields"]
            #[allow(deprecated)]
            pub const fn new(
                $($field_name: $field_ty,)*
                $( $($header_field: $header_ty,)* )?
                $( $($body_field_name: $body_field_ty),* )?
            ) -> Self {
                $name {
                    $($field_name,)*

                    $( $($header_field,)* )?

                    $( body: $body_name { $($body_field_name),* } )?
                }
            }
        }
    )*};
}

macro_rules! command_module {
    ($($vis:vis mod $mod:ident;)*) => {
        $($vis mod $mod;)*
        // TODO: Collect schemas from each object
    }
}

/*
// Experimental/incomplete alternate format, might reuse parts of it later
macro_rules! command2 {
    (
        // name, result and HTTP method
        $(#[$meta:meta])* struct $name:ident -> $result:ty: $method:ident(
            $($path:tt)* // will parse later
        )
        // permissions
        $(where $($kind:ident::$perm:ident)|+)?

        // HTTP Headers
        $($($(#[$header_meta:meta])* $header_name:literal => $header_vis:vis $header_field:ident: $header_ty:ty),+ $(,)*)?

        $({
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident: $field_ty:ty $(
                    // conditional additional permissions
                    where $($field_kind:ident::$field_perm:ident)|+ if $cond:expr
                )?
            ),+ $(,)*
        })?
    ) => {
        $(
            paste::paste! {
                #[doc = "Body struct for [" $name "]"]
                #[derive(Debug, Serialize, Deserialize)]
                pub struct [<$name Body>] {
                    $( $(#[$field_meta])* $field_vis $field_name: $field_ty ),+
                }
            }
        )?

        $(
            paste::paste! {
                #[doc = "Header struct for [" $name "]"]
                #[derive(Debug)]
                pub struct [<$name Headers>] {
                    $($(#[$header_meta])* $header_vis $header_field: $header_ty),+
                }
            }
        )?

        pub struct $name {

        }

        //impl $crate::api::command::sealed::Sealed for $name {}

        // type TEST = command2!(@BODY_TY $name: $($($field_name),+)?);
    };

    (@BODY_TY $name:ident: $($field_name:ident),+) => {paste::paste!([<$name Body>])};
    (@BODY_TY $name:ident: ) => {()};

    // final case
    (
        @BODY $(#[$meta:meta])* struct $name:ident {
            $($(#[$field_meta:meta])* $field_vis:vis $field_name:ident: $field_ty:ty),*
        }
        [] []
    ) => {
        $(#[$meta:meta])*
        pub struct $name {
            $( $(#[$field_meta])* $field_vis $field_name: $field_ty ),*
        }
    };

    (
        @BODY $(#[$meta:meta])* struct $name:ident {
            $($(#[$field_meta:meta])* $field_vis:vis $field_name:ident: $field_ty:ty),*
        }
        [$($param_name:ident: $param_ty:ty),+ $(/ $rest_params:tt)*]
        [$($rest_headers:tt)*]
    ) => {
        command2! {
            @BODY
            $(#[$meta:meta])*
            struct $name {
                $( $(#[$field_meta])* $field_vis $field_name: $field_ty ),*
            }
            [$($param_name:ident: $param_ty:ty),+]
            [$($rest:tt)*]
        }
    };
}
*/
