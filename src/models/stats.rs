use std::collections::HashMap;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Statistics {
    pub rooms: HashMap<Snowflake, RoomStatistics, super::Hasher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RoomStatistics {
    pub messages: u64,

    /// Total number of attachment files sent
    pub files: u64,

    /// If a prefix was given to the query, this will the message
    /// count where the message is prefixed by that.
    ///
    /// Otherwise it will be zero.
    pub prefixed: u64,
}
