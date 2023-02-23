use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "v")]
pub enum Embed {
    #[serde(rename = "1")]
    V1(EmbedV1),
}

pub mod v1;
pub use v1::*;

impl Embed {
    pub fn url(&self) -> Option<&str> {
        match self {
            Embed::V1(embed) => embed.url.as_ref().map(|x| x as _),
        }
    }
}
