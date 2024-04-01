use super::*;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MessageFlags: i32 {
        /// This message has been deleted
        const DELETED           = 1 << 0;
        /// This messages has been deleted by another user
        const REMOVED           = 1 << 1;
        /// If this message has children
        const PARENT            = 1 << 2;

        const MENTIONS_EVERYONE = 1 << 3;
        const MENTIONS_HERE     = 1 << 4;
        const TTS               = 1 << 5;

        const SUPRESS_EMBEDS    = 1 << 10;

        /// Set if the message has been starred by the user requesting it
        const STARRED           = 1 << 12;

        /// Top 6 bits are a language code,
        /// which is never actually exposed to users.
        const LANGUAGE          = 0b11_11_11 << (32 - 6);
    }
}

common::impl_serde_for_bitflags!(MessageFlags);
common::impl_schema_for_bitflags!(MessageFlags);

impl MessageFlags {
    #[inline]
    pub const fn from_bits_truncate_public(bits: i32) -> Self {
        Self::from_bits_truncate(bits).difference(Self::LANGUAGE)
    }
}

#[rustfmt::skip]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[derive(enum_primitive_derive::Primitive)]
#[repr(i16)]
pub enum MessageKind {
    #[default]
    Normal  = 0,
    Welcome = 1,
    Ephemeral = 2,
    Unavailable = 3,
}

common::impl_rkyv_for_pod!(MessageKind + CheckBytes);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct Message {
    pub id: Snowflake,
    pub room_id: Snowflake,
    pub party_id: Snowflake,

    #[serde(default, skip_serializing_if = "is_default")]
    pub kind: MessageKind,

    pub author: PartyMember,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<Snowflake>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<Timestamp>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<SmolStr>,

    pub flags: MessageFlags,

    #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
    #[cfg_attr(feature = "rkyv", with(rkyv::with::CopyOptimize))]
    pub pins: ThinVec<Snowflake>,

    #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
    #[cfg_attr(feature = "rkyv", with(rkyv::with::CopyOptimize))]
    pub user_mentions: ThinVec<Snowflake>,
    #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
    #[cfg_attr(feature = "rkyv", with(rkyv::with::CopyOptimize))]
    pub role_mentions: ThinVec<Snowflake>,
    #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
    #[cfg_attr(feature = "rkyv", with(rkyv::with::CopyOptimize))]
    pub room_mentions: ThinVec<Snowflake>,

    #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
    pub reactions: ThinVec<Reaction>,

    #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
    pub attachments: ThinVec<Attachment>,

    #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
    pub embeds: ThinVec<Embed>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub score: i32,
}

/// Simple enum for custom emote ids or emoji symbols
///
/// When written to URLs in the API (or `Display`ed), emojis become percent encoded, and custom emote ids
/// are prefixed with a colon (`:`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
#[serde(untagged)]
pub enum EmoteOrEmoji {
    Emote { emote: Snowflake },
    Emoji { emoji: SmolStr },
}

#[cfg(feature = "api")]
const _: () = {
    use super::*;

    use std::fmt;
    use std::str::FromStr;

    use percent_encoding::{percent_decode_str, percent_encode, NON_ALPHANUMERIC};

    impl fmt::Display for EmoteOrEmoji {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                EmoteOrEmoji::Emoji { emoji } => percent_encode(emoji.as_bytes(), NON_ALPHANUMERIC).fmt(f),
                EmoteOrEmoji::Emote { emote } => {
                    f.write_str(":")?;
                    emote.fmt(f)
                }
            }
        }
    }

    impl FromStr for EmoteOrEmoji {
        type Err = <Snowflake as FromStr>::Err;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // TODO: Better behavior for this?
            if s.is_empty() {
                return Ok(EmoteOrEmoji::Emoji {
                    emoji: SmolStr::default(),
                });
            }

            Ok(match s.as_bytes()[0] {
                b':' => EmoteOrEmoji::Emote { emote: s[1..].parse()? },
                _ => EmoteOrEmoji::Emoji {
                    emoji: percent_encoding::percent_decode_str(s).decode_utf8_lossy().into(),
                },
            })
        }
    }
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct ReactionShorthand {
    #[serde(flatten)]
    pub emote: EmoteOrEmoji,

    pub me: bool,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct ReactionFull {
    pub emote: EmoteOrEmoji,

    #[cfg_attr(feature = "rkyv", with(rkyv::with::CopyOptimize))]
    pub users: ThinVec<Snowflake>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
#[serde(untagged)]
pub enum Reaction {
    Shorthand(ReactionShorthand),
    Full(ReactionFull),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct Attachment {
    #[serde(flatten)]
    pub file: File,
}
