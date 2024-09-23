//! API Definitions

pub mod asset;
pub mod error;

#[macro_use]
mod command;

pub use command::{Command, CommandBody, CommandFlags, CommandResult, MissingItemError, RateLimit};

pub mod commands;

#[cfg(feature = "gateway")]
pub mod gateway;

/// Marker type for the presence of a valid authentication token
/// in the request headers.
///
/// This is checked when extracting commands from requests,
/// and must be inserted by the server when processing the request.
#[cfg(feature = "ftl")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuthMarker;
