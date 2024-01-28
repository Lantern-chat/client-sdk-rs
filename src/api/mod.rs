//! API Definitions

pub mod asset;
pub mod error;

#[macro_use]
mod command;

pub use command::{Command, CommandBody, CommandFlags, CommandResult, MissingItemError, RateLimit};

pub mod commands;

#[cfg(feature = "gateway")]
pub mod gateway;
