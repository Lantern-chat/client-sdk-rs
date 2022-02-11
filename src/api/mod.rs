//! API Definitions

pub mod error;

#[macro_use]
mod command;

pub use command::{Command, CommandFlags};

pub mod commands;
