use core::{fmt, time::Duration};
use std::num::NonZeroU64;

use http::{HeaderMap, Method};

pub(crate) mod sealed {
    pub trait Sealed {}
}

use crate::models::Permissions;

bitflags2! {
    /// Flags for command functionality.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CommandFlags: u8 where "command" {
        /// Command requires authorization to execute.
        const AUTHORIZED    = 1 << 0;

        /// Command has a request body.
        const HAS_BODY      = 1 << 1;
        const HAS_RESPONSE  = 1 << 2;
        const STREAMING     = 1 << 3;

        const BOTS_ONLY     = 1 << 5;
        const USERS_ONLY    = 1 << 6;
        const ADMIN_ONLY    = 1 << 7;
    }
}

#[allow(unused)]
impl CommandFlags {
    // easier to declare in the macro
    pub(crate) const B: Self = Self::BOTS_ONLY;
    pub(crate) const U: Self = Self::USERS_ONLY;
    pub(crate) const A: Self = Self::ADMIN_ONLY;
}

impl_rkyv_for_bitflags!(pub CommandFlags: u8);
impl_schema_for_bitflags!(CommandFlags);

/// Rate-limiting configuration for a command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimit {
    /// Ideal time between each request
    pub emission_interval: Duration,

    /// Maximum number of requests that can be made in a burst, before rate-limiting kicks in.
    pub burst_size: NonZeroU64,
}

impl RateLimit {
    /// Default rate-limit config for commands when not otherwise specified.
    ///
    /// ```ignore
    /// RateLimit {
    ///     emission_interval: 50ms,
    ///     burst_size: 5,
    /// }
    /// ```
    ///
    /// Or approximately 20 requests per second, with up to 5 requests burst within the `emission_interval`,
    /// but the client must wait for them to replenish before another burst.
    pub const DEFAULT: RateLimit = RateLimit {
        emission_interval: Duration::from_millis(50),
        burst_size: unsafe { NonZeroU64::new_unchecked(5) },
    };
}

impl Default for RateLimit {
    #[inline]
    fn default() -> Self {
        RateLimit::DEFAULT
    }
}

/// Combined trait for serde and rkyv functionality
#[cfg(feature = "rkyv")]
pub trait CommandResult: Send + serde::de::DeserializeOwned + serde::ser::Serialize + rkyv::Archive {}

/// Combined trait for serde and rkyv functionality
#[cfg(feature = "rkyv")]
pub trait CommandBody: Send + serde::ser::Serialize + rkyv::Archive {}

#[cfg(feature = "rkyv")]
impl<T> CommandResult for T where T: Send + serde::de::DeserializeOwned + serde::ser::Serialize + rkyv::Archive {}

#[cfg(feature = "rkyv")]
impl<T> CommandBody for T where T: Send + serde::ser::Serialize + rkyv::Archive {}

/// Combined trait for serde and rkyv functionality
#[cfg(not(feature = "rkyv"))]
pub trait CommandResult: Send + serde::de::DeserializeOwned + serde::ser::Serialize {}

/// Combined trait for serde and rkyv functionality
#[cfg(not(feature = "rkyv"))]
pub trait CommandBody: Send + serde::ser::Serialize {}

#[cfg(not(feature = "rkyv"))]
impl<T> CommandResult for T where T: Send + serde::de::DeserializeOwned + serde::ser::Serialize {}

#[cfg(not(feature = "rkyv"))]
impl<T> CommandBody for T where T: Send + serde::ser::Serialize {}

/// Error returned when an item is missing from a stream or the stream is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingItemError;

impl fmt::Display for MissingItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Missing Item Error")
    }
}

impl core::error::Error for MissingItemError {}

/// Client Command, tells the client to perform specific requests
///
/// A "Command" is a mid-level abstraction around REST endpoints and their bodies. Not perfect,
/// but zero-cost and simple. Other abstractions can be built on top of it.
///
/// A command consists of three parts: the URL, the "body", and headers.
///
/// For the case of `GET`/`OPTIONS` commands, the body becomes query parameters.
pub trait Command: sealed::Sealed {
    /// Whether the command has a query string or sends a body
    const IS_QUERY: bool;

    /// The underlying type of each returned item, be it one or many.
    type Item: CommandResult;

    /// Item(s) returned from the server by a given command
    type Result: CommandResult;

    /// Body type for the command
    type Body: CommandBody;

    /// HTTP Method used to execute the command
    const HTTP_METHOD: Method;

    /// Flags for the command, defaults to empty.
    const FLAGS: CommandFlags;

    /// Baseline rate-limiting parameters, defaults to [`RateLimit::DEFAULT`].
    ///
    /// The server may choose to adapt this as needed, and
    /// it may not be the only rate-limiting factor depending
    /// on the request.
    const RATE_LIMIT: RateLimit;

    /// On the server side, how long to wait before timing out the request.
    const SERVER_TIMEOUT: Duration;

    /// Path pattern for the command (without query) when used with matchit 0.8 or higher.
    const ROUTE_PATTERN: &'static str;

    /// Serialize/format the REST path (without query)
    fn format_path<W: fmt::Write>(&self, w: W) -> fmt::Result;

    /// Serialize/format the REST path (without query) and return it as a string
    fn path(&self) -> Result<String, fmt::Error> {
        let mut buf = String::new();
        self.format_path(&mut buf)?;
        Ok(buf)
    }

    /// Body to be serialized as request body or query parameters (if GET)
    fn body(&self) -> &Self::Body;

    /// Used to collect the [`Result`](Self::Result) from an arbitrary [`Stream`](futures::Stream) of items.
    fn collect<S, E>(stream: S) -> impl ::core::future::Future<Output = Result<Self::Result, E>> + Send
    where
        S: futures::Stream<Item = Result<Self::Item, E>> + Send,
        E: From<MissingItemError>;

    /// Hint given to preallocate body size, only used for query strings
    #[inline(always)]
    fn body_size_hint(&self) -> usize {
        0
    }

    /// Computes required permissions
    fn perms(&self) -> Permissions;

    /// Insert any additional headers required to perform this command
    #[inline(always)]
    fn add_headers(&self, _map: &mut HeaderMap) {}

    #[cfg(feature = "schema")]
    /// Generate a schema for this command
    fn schema(gen: &mut schemars::gen::SchemaGenerator) -> (String, okapi::openapi3::PathItem);
}

/// Takes an expression like: "a" / value / "b" / value2
/// and converts it into a sequence of `Write` writes
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

/// Takes a path pattern like: "a" / value / "b" / value2
/// and converts it into a static string for use in matchit 0.8 or higher
macro_rules! static_path_pattern {
    ([$($value:literal),+] [/ $next:literal $(/ $tail:tt)*]) => {
        static_path_pattern!([$($value,)+ $next] [$(/ $tail)*])
    };

    ([$($value:literal),+] [/ $next:tt $(/ $tail:tt)*]) => {
        concat![$("/", $value,)+ static_path_pattern!([$next] [$(/ $tail)*])]
    };

    ([$value:ident] [/ $next:tt $(/ $tail:tt)*]) => {
        concat!["/{", stringify!($value), "}", static_path_pattern!([$next] [$(/ $tail)*])]
    };

    ([$($value:literal),*] []) => { concat![$("/", $value),*] };
    ([$value:ident] []) => { concat!["/{", stringify!($value), "}"] };
}

/// Like `static_path_pattern!`, but for path template strings in TypeScript
#[cfg(feature = "ts")]
macro_rules! ts_path {
    ([$($value:literal),+] [/ $next:literal $(/ $tail:tt)*]) => {
        ts_path!([$($value,)+ $next] [$(/ $tail)*])
    };

    ([$($value:literal),+] [/ $next:tt $(/ $tail:tt)*]) => {
        concat![$("/", $value,)+ ts_path!([$next] [$(/ $tail)*])]
    };

    ([$value:ident] [/ $next:tt $(/ $tail:tt)*]) => {
        concat!["/${this.", stringify!($value), "}", ts_path!([$next] [$(/ $tail)*])]
    };

    ([$($value:literal),*] []) => { concat![$("/", $value),*] };
    ([$value:ident] []) => { concat!["/${this.", stringify!($value), "}"] };
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

    (@STREAMING One) => { CommandFlags::empty() };
    (@STREAMING Many) => { CommandFlags::STREAMING };
    (@STREAMING $other:ident) => { compile_error!("Must use One or Many for Command result") };

    (@AGGREGATE One $ty:ty) => { $ty };
    (@AGGREGATE Many $ty:ty) => { Vec<$ty> };

    (@COLLECT One) => {
        async fn collect<S, E>(stream: S) -> Result<Self::Result, E>
        where
            S: futures::Stream<Item = Result<Self::Item, E>> + Send,
            E: From<MissingItemError>,
        {
            let mut stream = core::pin::pin!(stream);

            use futures::StreamExt;

            match stream.next().await {
                Some(item) => item,
                None => Err(E::from(MissingItemError)),
            }
        }
    };

    (@COLLECT Many) => {
        async fn collect<S, E>(stream: S) -> Result<Self::Result, E>
        where
            S: futures::Stream<Item = Result<Self::Item, E>> + Send,
            E: From<MissingItemError>,
        {
            let mut stream = core::pin::pin!(stream);

            use futures::StreamExt;

            let mut items = Vec::new();
            while let Some(item) = stream.next().await {
                items.push(item?);
            }

            Ok(items)
        }
    };

    // entry point
    ($group_name:ident; $(
        // meta
        $(#[$($meta:tt)*])*

        // two symbols to differentiate auth and noauth commands (keyword struct verified in @STRUCT)
        $(+$auth_struct:ident)? $(-$noauth_struct:ident)?

        // name, result and HTTP method
        $name:ident $(($($flag:ident)|*))? -> $count:ident $result:ty: $method:ident$([$emission_interval:literal ms $(, $burst_size:literal)?])?(
            $head:tt $(/ $tail:tt)*
        )
        // permissions
        $(where $($perm:ident)|+)?

        // HTTP Headers
        $($($(#[$header_meta:meta])* $header_name:literal => $header_vis:vis $header_field:ident: $header_ty:ty),+ $(,)*)?

        // fields
        {
            $(
                $(#[$($field_meta:tt)*])*
                $field_vis:vis $field_name:ident: $field_ty:ty $(
                    // conditional additional permissions
                    where $($field_perm:ident)|+ if $cond:expr
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
                            where $($body_field_perm:ident)|+ if $body_field_cond:expr
                        )?

                    ),* $(,)*
                }
            )?
        }
    )*) => {paste::paste!{

    #[cfg(feature = "ts")]
    #[allow(unused_variables)]
    pub fn [<register_ $group_name:snake _routes>](registry: &mut ts_bindgen::TypeRegistry) {
        $( <$name as ts_bindgen::TypeScriptDef>::register(registry); )*
    }

    $(
        // verify presence of exactly one `struct` without prefix
        command!(@STRUCT $($auth_struct)? $($noauth_struct)?);

        impl $crate::api::command::sealed::Sealed for $name {}
        impl $crate::api::command::Command for $name {
            const IS_QUERY: bool = matches!(
                http::Method::$method,
                http::Method::GET | http::Method::OPTIONS | http::Method::HEAD | http::Method::CONNECT | http::Method::TRACE
            );

            command!(@COLLECT $count);

            type Item = $result;

            type Result = command!(@AGGREGATE $count $result);

            const HTTP_METHOD: http::Method = http::Method::$method;

            const FLAGS: CommandFlags = CommandFlags::empty().union(command!(@STREAMING $count))
                .union(const {
                    if size_of::<$result>() != 0 {
                        CommandFlags::HAS_RESPONSE
                    } else {
                        CommandFlags::empty()
                    }
                })
                $(.union((stringify!($body_name), CommandFlags::HAS_BODY).1))?
                $(.union((stringify!($auth_struct), CommandFlags::AUTHORIZED).1))?
                $( $(.union(CommandFlags::$flag))* )?
            ;

            $(
                #[doc = "```ignore\nRateLimit {\n    emission_interval: " $emission_interval "ms,\n"]
                $(#[doc = "    burst_size: " $burst_size ","])?
                #[doc = "}\n```\nIf not specified, the `burst_size` will be from [`RateLimit::DEFAULT`]."]
            )?
            #[allow(clippy::needless_update)]
            const RATE_LIMIT: RateLimit = RateLimit {
                $(emission_interval: core::time::Duration::from_millis($emission_interval),
                $(burst_size: {
                    assert!($burst_size > 0, "Burst Size must be nonzero!");
                    unsafe { ::core::num::NonZeroU64::new_unchecked($burst_size) }
                }, )?)?
                ..RateLimit::DEFAULT
            };

            const SERVER_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

            #[allow(unused_mut, unused_variables, deprecated)]
            fn perms(&self) -> Permissions {
                let mut base = crate::perms!($($($perm)|+)?);

                let $name {
                    $(ref $field_name,)*

                    $( $(ref $header_field,)* )?

                    $(
                        body: $body_name { $(ref $body_field_name),* }
                    )?
                } = self;

                $($(
                    if $cond {
                        base |= crate::perms!($(Permissions::$field_perm)|+)
                    }
                )?)*

                base
            }

            const ROUTE_PATTERN: &'static str = static_path_pattern!(["api", "v1", $head] [$(/ $tail)*]);

            #[inline]
            #[allow(deprecated)]
            fn format_path<W: core::fmt::Write>(&self, mut w: W) -> core::fmt::Result {
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

                path_item.[<$method:lower>] = Some({
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

                // TODO: Self::ROUTE_PATTERN.to_owned() instead?
                (static_path_pattern!([$head] [$(/ $tail)*]).to_owned(), path_item)
            }
        }

        #[derive(Debug)]
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
        #[must_use = "Commands do nothing unless executed via a `Driver`."]
        $(#[$($meta)*])*
        pub struct $name {
            $($(#[$($field_meta)*])* $field_vis $field_name: $field_ty, )*

            $( $($(#[$header_meta])* $header_vis $header_field: $header_ty, )* )?

            $(
                /// Body to be serialized as request body or query parameters (if GET)
                pub body: $body_name,
            )?
        }

        #[cfg(feature = "ts")]
        const _: () = {
            use ts_bindgen::{TypeRegistry, TypeScriptDef, TypeScriptType};

            impl TypeScriptDef for $name {
                fn register(registry: &mut TypeRegistry) -> TypeScriptType {
                    use crate::api::Command;

                    if registry.contains(stringify!($name)) {
                        return TypeScriptType::Named(stringify!($name));
                    }

                    let TypeScriptType::Named(cmd_flags) = CommandFlags::register(registry) else {
                        panic!("Failed to register CommandFlags as expected!");
                    };

                    #[allow(unused_mut, clippy::vec_init_then_push)]
                    let ty = TypeScriptType::ApiDecl {
                        command_flags: {
                            let mut flags = Vec::new();

                            for (name, _) in Self::FLAGS.iter_names() {
                                flags.push(TypeScriptType::EnumValue(cmd_flags, name));
                            }

                            flags
                        },
                        name: stringify!($name).into(),
                        method: Self::HTTP_METHOD.as_str().to_owned().into(),
                        body_type: None $(.or(Some(Box::new($body_name::register(registry)))))?,
                        return_type: Box::new(<$result>::register(registry)),

                        path: ts_path!([$head] [$(/ $tail)*]),

                        form_type: {
                            let mut fields = Vec::new();

                            $(
                                fields.push((
                                    stringify!($field_name).into(),
                                    $field_ty::register(registry),
                                    concat!($(command!(@DOC #[$($field_meta)*])),*).trim().into(),
                                ));
                            )*

                            $(
                                fields.push((
                                    "body".into(),
                                    $body_name::register(registry),
                                    "".into(),
                                ));
                            )?

                            Box::new(TypeScriptType::interface(fields, 0))
                        }
                    };

                    registry.insert(stringify!($name), ty, concat!($(command!(@DOC #[$($meta)*])),*).trim());

                    registry.tag(stringify!($name), "command");

                    TypeScriptType::Named(stringify!($name))
                }
            }
        };

        #[cfg(feature = "rkyv")]
        const _: () = {
            const fn assert_archive<T: rkyv::Archive>() {}
            assert_archive::<$name>();
        };

        $(
            #[derive(Debug, Serialize, Deserialize)]
            #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
            #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
            #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "command"))]
            $(#[$body_meta])*
            pub struct $body_name {
                $( $(#[$($body_field_meta)*])* $body_field_vis $body_field_name: $body_field_ty ),*
            }

            impl ::core::ops::Deref for $name {
                type Target = $body_name;

                #[inline]
                fn deref(&self) -> &Self::Target {
                    &self.body
                }
            }

            impl ::core::ops::DerefMut for $name {
                #[inline]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.body
                }
            }
        )?

        impl $name {
            #[doc = "Construct new instance from individual fields"]
            #[allow(deprecated, clippy::too_many_arguments, clippy::new_without_default)]
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

        #[cfg(feature = "ftl")]
        const _: () = {
            use ftl::{Request, extract::{FromRequest, FromRequestParts, path::Path}};

            mod segments {
                ftl::path_segment! {
                    $(pub [<$field_name:camel>]: super::[<$field_ty>],)*
                }
            }

            impl<S> FromRequest<S> for $name
                where S: Send + Sync,
            {
                type Rejection = ftl::Error;

                #[allow(unused_variables, clippy::manual_async_fn)]
                fn from_request(req: Request, state: &S) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
                    async move {
                        let (mut parts, body) = req.into_parts();

                        if parts.method != <Self as $crate::api::Command>::HTTP_METHOD {
                            return Err(ftl::Error::MethodNotAllowed);
                        }

                        $(
                            _ = stringify!($auth_struct);

                            if parts.extensions.get::<crate::api::AuthMarker>().is_none() {
                                return Err(ftl::Error::Unauthorized);
                            }
                        )?

                        let Path(($($field_name,)*)) = Path::<($(segments::[<$field_name:camel>],)*)>::from_request_parts(&mut parts, state).await.map_err(ftl::Error::from)?;

                        Ok($name {
                            $($field_name,)*

                            $(body: if <Self as $crate::api::Command>::IS_QUERY {
                                ftl::extract::query::Query::<$body_name>::from_request_parts(&mut parts, state).await.map_err(ftl::Error::from)?.0
                            } else {
                                ftl::extract::one_of::OneOfAny::<$body_name>::from_request(Request::from_parts(parts, body), state).await.map_err(ftl::Error::from)?.0
                            })?
                        })
                    }
                }
            }
        };
    )*}};
}

macro_rules! command_module {
    ($($vis:vis mod $mod:ident;)*) => {
        $($vis mod $mod;)*

        pub mod all {
            $($vis use super::$mod::*;)*
        }

        #[cfg(feature = "ts")]
        pub fn register_routes(registry: &mut ts_bindgen::TypeRegistry) {
            paste::paste! { $( $mod::[<register_ $mod _routes>](registry); )* }
        }

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
