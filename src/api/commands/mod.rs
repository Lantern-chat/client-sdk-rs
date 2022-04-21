use crate::{api::command::CommandFlags, models::*};
use smol_str::SmolStr;

command_module! {
    pub mod file;
    pub mod party;
    pub mod room;
    pub mod user;
}
