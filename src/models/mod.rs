//! Object data structures used within Lantern

#![allow(unused_imports, clippy::identity_op)]
#![deny(unused_imports)]
//#![cfg_attr(debug_assertions, warn(missing_docs))]

#[cfg(feature = "rkyv")]
use rkyv::with::Niche;

pub use smol_str::SmolStr;
pub use thin_vec::ThinVec;
pub use timestamp::Timestamp;
pub use triomphe::Arc;
pub use util::fixed_str::FixedStr;

#[macro_use]
pub mod util;
pub mod nullable;
pub mod sf;

pub use nullable::Nullable;
pub use sf::Snowflake;

/// Defines Snowflake aliases to easier keep track of what ID is for what.
pub mod aliases {
    use super::Snowflake;

    macro_rules! decl_aliases {
        ($($(#[$meta:meta])* $name:ident,)*) => {
            $(
                $(#[$meta])*
                pub type $name = Snowflake;

                #[cfg(feature = "rkyv")]
                paste::paste! {
                    #[doc = "Archived version of [`" $name "`]"]
                    pub type [<Archived $name>] = rkyv::Archived<Snowflake>;
                }
            )*
        };
    }

    decl_aliases! {
        /// Snowflake ID for a Party
        PartyId,
        /// Snowflake ID for a User
        UserId,
        /// Snowflake ID for a Role
        RoleId,
        /// Snowflake ID for a Room
        RoomId,
        /// Snowflake ID for a Message
        MessageId,
        /// Snowflake ID for a Custom Emote
        EmoteId,
        /// Snowflake ID for a File
        FileId,
        /// Snowflake ID for an party Invite
        InviteId,
        /// Snowflake ID for a message Thread
        ThreadId,
        /// Snowflake ID for a Pin Folder
        FolderId,
    }
}

pub use aliases::*;

macro_rules! decl_newtype_prefs {
    ($( $(#[$meta:meta])* $name:ident: $ty:ty $(= $default:expr)?,)*) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
            #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize), archive(check_bytes))]
            #[repr(transparent)]
            pub struct $name(pub $ty);

            $(
                impl Default for $name {
                    #[inline(always)]
                    fn default() -> Self {
                        $name($default.into())
                    }
                }
            )?

            impl core::ops::Deref for $name {
                type Target = $ty;

                #[inline(always)]
                fn deref(&self) -> &$ty {
                    &self.0
                }
            }

            impl core::ops::DerefMut for $name {
                #[inline(always)]
                fn deref_mut(&mut self) -> &mut $ty {
                    &mut self.0
                }
            }
        )*
    };
}

pub mod asset;
pub mod auth;
pub mod config;
pub mod embed;
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
pub mod stats;
pub mod thread;
pub mod user;

pub type Hasher = rustc_hash::FxBuildHasher;

pub use self::{
    asset::*, auth::*, config::*, embed::*, emote::*, file::*, gateway::*, invite::*, message::*, party::*, permission::*,
    presence::*, role::*, room::*, session::*, sf::*, stats::*, thread::*, user::*,
};

/// Directional search query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    archive(check_bytes)
)]
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
