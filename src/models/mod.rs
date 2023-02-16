//! Object data structures used within Lantern

#![allow(unused_imports)]

pub use smol_str::SmolStr;
pub use timestamp::Timestamp;

pub mod nullable;

pub use nullable::Nullable;

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
pub mod prefs;
pub mod presence;
pub mod role;
pub mod room;
pub mod session;
pub mod sf;
pub mod thread;
pub mod user;

use crate::util::fixed::FixedStr;

pub use self::{
    asset::*, auth::*, config::*, embed::*, emote::*, file::*, gateway::*, invite::*, message::*, party::*,
    permission::*, prefs::*, presence::*, role::*, room::*, session::*, sf::*, thread::*, user::*,
};

/// Directional search query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
