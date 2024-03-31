//! Asset handling utilities and types

use smol_str::SmolStr;

pub use crate::models::AssetFlags;

/// When fetching assets, this query can be used to specify the desired asset format.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum AssetQuery {
    /// Simple and efficient query using bitflags, e.g. `?f=1234`
    Flags {
        /// Bitfield of flags to filter by
        #[serde(alias = "f")]
        flags: u16,
    },
    /// Human-readable query, e.g. `?q=100&a=1&t=1&ext=png`
    HumanReadable {
        /// Quality of the asset from 0-100
        #[serde(alias = "q")]
        quality: u8,
        /// Whether to select animated assets
        #[serde(alias = "a")]
        animated: bool,
        /// Whether to select assets with alpha channels
        #[serde(alias = "t")]
        with_alpha: bool,
        /// File extension to filter by if picking a specific format
        #[serde(skip_serializing_if = "Option::is_none")]
        ext: Option<SmolStr>,
    },
}

impl From<AssetFlags> for AssetQuery {
    fn from(flags: AssetFlags) -> Self {
        AssetQuery::Flags {
            flags: flags.bits() as u16,
        }
    }
}

impl From<AssetQuery> for AssetFlags {
    fn from(query: AssetQuery) -> Self {
        match query {
            AssetQuery::Flags { flags } => AssetFlags::from_bits_truncate(flags as i16),
            AssetQuery::HumanReadable {
                quality,
                animated,
                with_alpha,
                ext,
            } => {
                let mut flags = AssetFlags::empty().with_quality(quality).with_alpha(with_alpha);

                if animated {
                    flags |= AssetFlags::ANIMATED;
                }

                match ext {
                    Some(ext) => flags.union(AssetFlags::from_ext(&ext)),
                    None => flags.union(AssetFlags::FORMATS).difference(AssetFlags::MAYBE_UNSUPPORTED_FORMATS),
                }
            }
        }
    }
}
