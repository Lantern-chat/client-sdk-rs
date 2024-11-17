use crate::{
    api::command::{CommandFlags, MissingItemError, RateLimit},
    models::*,
};
use smol_str::SmolStr;

command_module! {
    pub mod config;
    pub mod file;
    pub mod party;
    pub mod room;
    pub mod user;
    pub mod invite;
}
