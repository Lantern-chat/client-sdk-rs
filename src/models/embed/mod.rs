// This file is dual-licensed under either the MIT or Apache 2.0 license.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// You may choose either license to govern your use of this file.
// Any types re-exported from this file also fall under the same license.

//! Message Embed Structures, dual licensed under MIT or Apache 2.0

use super::*;

/// Default type returned by the embed server
///
/// You probably want to deserialize the payloads with this type alias
pub type EmbedWithExpire = (timestamp::Timestamp, Embed);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
#[serde(tag = "v")]
pub enum Embed {
    #[serde(rename = "1")]
    V1(EmbedV1),
}

pub mod v1;
pub use v1::*;

impl Embed {
    #[must_use]
    pub fn url(&self) -> Option<&str> {
        match self {
            Embed::V1(embed) => embed.url.as_ref().map(|x| x as _),
        }
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "rkyv")]
    #[test]
    fn test_rkyv() {
        use rkyv::{rancor::Error, Archived};

        _ = rkyv::access::<Archived<Embed>, Error>(&[]);
    }
}
