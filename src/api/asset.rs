use smol_str::SmolStr;

pub use crate::models::AssetFlags;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum AssetQuery {
    Flags {
        flags: u16,
    },
    HumanReadable {
        quality: u8,
        animated: bool,
        with_alpha: bool,
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
