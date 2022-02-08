use std::fmt;

use http::Method;

pub(crate) mod sealed {
    pub trait Sealed {}
}

use crate::models::Permission;

pub trait Command: sealed::Sealed + serde::Serialize {
    type Result;

    const METHOD: Method;
    const PERMS: Permission;

    /// Serialize/format the URI path (with query)
    fn format_path<W: fmt::Write>(&self, w: W) -> fmt::Result;
}

// Macro to autogenerate most command trait implementations.
macro_rules! command {
    // munchers
    (@seg $w:expr, $this:expr, [$value:literal] [/ $next:tt $(/ $tail:tt)*]) => {
        $w.write_str(concat!($value, "/"))?;
        command!(@seg $w, $this, [$next] [$(/ $tail)*]);
    };

    (@seg $w:expr, $this:expr, [$value:ident] [/ $next:tt $(/ $tail:tt)*]) => {
        write!($w, "{}/", $this.$value)?;
        command!(@seg $w, $this, [$next] [$(/ $tail)*]);
    };

    (@seg $w:expr, $this:expr, [$value:literal] []) => { $w.write_str($value)?; };
    (@seg $w:expr, $this:expr, [$value:ident] []) => { write!($w, "{}", $this.$value)?; };

    // entry point
    ($(
        // name, result and HTTP method
        $(#[$meta:meta])* struct $name:ident -> $result:ty: $method:ident(
            $head:tt $(/ $tail:tt)* $(? $($($query_alias:literal)? $query:ident)&+ )?
        )
        // permissions
        where $($kind:ident::$perm:ident)|*
        // fields
        $({
            $( $(#[$field_meta:meta])* $field_vis:vis $field_name:ident: $field_ty:ty ),* $(,)*
        })?
    )*) => {$(
        impl $crate::api::command::sealed::Sealed for $name {}
        impl $crate::api::command::Command for $name {
            type Result = $result;

            const METHOD: http::Method = http::Method::$method;
            const PERMS: Permission = crate::perms!($($kind::$perm)|*);

            fn format_path<W: std::fmt::Write>(&self, mut w: W) -> std::fmt::Result {
                command!(@seg w, self, [$head] [$(/ $tail)*]);

                $(
                    use form_urlencoded::Serializer as UrlEncodedSerializer;
                    use serde::ser::{Serializer, SerializeStruct};

                    const LEN: usize = 0 $(+ (stringify!($query), 1).1)*;

                    // preallocate with number of equal signs + lengths of keys
                    let mut encoder = UrlEncodedSerializer::new(String::with_capacity(
                        LEN $(+ [$($query_alias,)? stringify!($query)][0].len())*
                    ));
                    let serializer = serde_urlencoded::Serializer::new(&mut encoder);

                    let mut s = serializer.serialize_struct(stringify!($name), LEN).map_err(|_| std::fmt::Error)?;
                    $( s.serialize_field([$($query_alias,)? stringify!($query)][0], &self.$query).map_err(|_| std::fmt::Error)?;)*

                    s.end().map_err(|_| std::fmt::Error)?;

                    let params = encoder.finish();

                    if !params.is_empty() {
                        w.write_str("?")?;
                        w.write_str(&params)?;
                    }
                )?

                Ok(())
            }
        }

        $(#[$meta])*
        #[derive(Debug, Serialize)]
        pub struct $name {
            $( $($(#[$field_meta])* $field_vis $field_name: $field_ty),* )?
        }
    )*};
}
