use crate::{
    api::command::{CommandFlags, MissingItemError, RateLimit},
    models::*,
};
use smol_str::SmolStr;

// NOTE: This macro is here to aggregate schema definitions, but otherwise does very little.
command_module! {
    pub mod config;
    pub mod file;
    pub mod party;
    pub mod room;
    pub mod user;
    pub mod invite;
}
