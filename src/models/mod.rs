//! Object data structures used within Lantern

#![allow(unused_imports, clippy::identity_op)]

use common::fixed::FixedStr;

#[cfg(feature = "rkyv")]
use rkyv::with::Niche;

pub use smol_str::SmolStr;
pub use thin_vec::ThinVec;
pub use timestamp::Timestamp;
pub use triomphe::Arc;

pub mod nullable;

pub use nullable::Nullable;

pub mod embed {
    pub use ::embed::{
        v1, BoxedEmbedMedia, Embed, EmbedAuthor, EmbedField, EmbedFlags, EmbedFooter, EmbedMedia, EmbedProvider, EmbedType,
        EmbedV1, UrlSignature,
    };
}

macro_rules! decl_newtype_prefs {
    ($( $(#[$meta:meta])* $name:ident: $ty:ty $(= $default:expr)?,)*) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
            #[cfg_attr(feature = "rkyv", derive(rkyv::CheckBytes))]
            #[repr(transparent)]
            pub struct $name(pub $ty);

            $(
                impl Default for $name {
                    fn default() -> Self {
                        $name($default.into())
                    }
                }
            )?

            impl core::ops::Deref for $name {
                type Target = $ty;

                fn deref(&self) -> &$ty {
                    &self.0
                }
            }

            common::impl_rkyv_for_pod!($name);
        )*
    };
}

pub mod asset;
pub mod auth;
pub mod config;
pub mod emote;
pub mod file;
pub mod gateway;
pub mod invite;
pub mod message;
pub mod party;
pub mod permission;
pub mod presence;
pub mod role;
pub mod room;
pub mod session;
pub mod sf;
pub mod stats;
pub mod thread;
pub mod user;

#[cfg(not(feature = "ahash"))]
type Hasher = std::collections::hash_map::RandomState;

#[cfg(feature = "ahash")]
type Hasher = ahash::RandomState;

pub use self::{
    asset::*, auth::*, config::*, embed::*, emote::*, file::*, gateway::*, invite::*, message::*, party::*, permission::*,
    presence::*, role::*, room::*, session::*, sf::*, stats::*, thread::*, user::*,
};

/// Directional search query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
#[serde(rename_all = "lowercase")]
pub enum Cursor {
    Exact(Snowflake),
    After(Snowflake),
    Before(Snowflake),
}

#[allow(unused)]
#[inline]
pub(crate) const fn is_false(value: &bool) -> bool {
    !*value
}

#[allow(unused)]
#[inline]
pub(crate) const fn is_true(value: &bool) -> bool {
    *value
}

#[allow(unused)]
#[inline]
pub(crate) fn is_none_or_empty<T: IsEmpty>(value: &Option<T>) -> bool {
    match value {
        None => true,
        Some(v) => v._is_empty(),
    }
}

#[allow(unused)]
#[inline]
pub(crate) fn default_true() -> bool {
    true
}

#[allow(unused)]
#[inline]
pub(crate) fn is_default<T>(value: &T) -> bool
where
    T: Default + PartialEq,
{
    *value == T::default()
}

//

pub(crate) trait IsEmpty {
    fn _is_empty(&self) -> bool;
}

impl<T> IsEmpty for &[T] {
    #[inline]
    fn _is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmpty for Vec<T> {
    #[inline]
    fn _is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmpty for ThinVec<T> {
    #[inline]
    fn _is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl IsEmpty for SmolStr {
    #[inline]
    fn _is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl IsEmpty for String {
    #[inline]
    fn _is_empty(&self) -> bool {
        self.is_empty()
    }
}
