use super::*;

// TODO: no_std map of some kind
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct Statistics {
    pub rooms: HashMap<RoomId, RoomStatistics, FxRandomState2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
pub struct RoomStatistics {
    /// Total number of messages sent
    pub messages: u64,

    /// Total number of attachment files sent
    pub files: u64,

    /// If a prefix was given to the query, this will the message
    /// count where the message is prefixed by that.
    ///
    /// Otherwise it will be zero.
    pub prefixed: u64,
}
