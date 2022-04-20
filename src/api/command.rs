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
    fn body_size_hint(&self) -> usize {
        0
    }

    /// Computes required permissions
    fn perms(&self) -> Permission;

    /// Insert any additional headers required to perform this command
    #[inline(always)]
    fn add_headers(&self, _map: &mut HeaderMap) {}
}

// Macro to autogenerate most Command trait implementations.
macro_rules! command {
    // munchers
    (@seg $w:expr, $this:expr, [$($value:literal),+] [/ $next:literal $(/ $tail:tt)*]) => {
        command!(@seg $w, $this, [$($value,)+ $next] [$(/ $tail)*]);
    };

    (@seg $w:expr, $this:expr, [$($value:literal),+] [/ $next:tt $(/ $tail:tt)*]) => {
        $w.write_str(concat!($("/", $value),+))?;
        command!(@seg $w, $this, [$next] [$(/ $tail)*]);
    };

    (@seg $w:expr, $this:expr, [$value:ident] [/ $next:tt $(/ $tail:tt)*]) => {
        write!($w, "/{}", $this.$value)?;
        command!(@seg $w, $this, [$next] [$(/ $tail)*]);
    };

    (@seg $w:expr, $this:expr, [$($value:literal),*] []) => { $w.write_str(concat!($("/", $value),*))?; };
    (@seg $w:expr, $this:expr, [$value:ident] []) => { write!($w, "{}", $this.$value)?; };

    (@STRUCT struct) => {};

    (@BODY_TY $name:ident) => { $name };
    (@BODY_TY) => { () };

    (@BODY_RETURN $name:ident $ret:expr) => { $ret };
    (@BODY_RETURN ) => { &() };

    // entry point
    ($(
        // meta
        $(#[$meta:meta])*

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
                $(#[$field_meta:meta])*
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

                        $(#[$body_field_meta:meta])*
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

            #[allow(unused_mut, unused_variables)]
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
            fn format_path<W: std::fmt::Write>(&self, mut w: W) -> std::fmt::Result {
                command!(@seg w, self, [$head] [$(/ $tail)*]);

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
        }

        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name {
            $($(#[$field_meta])* $field_vis $field_name: $field_ty, )*

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
                $( $(#[$body_field_meta])* $body_field_vis $body_field_name: $body_field_ty ),*
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
