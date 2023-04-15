use super::*;

bitflags::bitflags! {
    pub struct MessageFlags: i16 {
        const DELETED           = 1 << 0;
        const MENTIONS_EVERYONE = 1 << 1;
        const MENTIONS_HERE     = 1 << 2;
        const TTS               = 1 << 3;
        const SUPRESS_EMBEDS    = 1 << 4;
        const HAS_LINK          = 1 << 5;

        /// Top 6 bits are a language code,
        /// which is never actually exposed to users.
        const LANGUAGE          = 0b111111 << (16 - 6);
    }
}

serde_shims::impl_serde_for_bitflags!(MessageFlags);
impl_schema_for_bitflags!(MessageFlags);

impl MessageFlags {
    #[inline]
    pub const fn from_bits_truncate_public(bits: i16) -> Self {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Message {
    pub id: Snowflake,
    pub room_id: Snowflake,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party_id: Option<Snowflake>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub kind: MessageKind,

    pub author: User,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<PartialPartyMember>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<Snowflake>,

    pub created_at: Timestamp,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<Timestamp>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<SmolStr>,

    pub flags: MessageFlags,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pins: Vec<Snowflake>,

    /// True if the message has been starred by the current user
    #[serde(default, skip_serializing_if = "is_false")]
    pub starred: bool,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_mentions: Vec<Snowflake>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub role_mentions: Vec<Snowflake>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub room_mentions: Vec<Snowflake>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reactions: Vec<Reaction>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<Embed>,
}

/// Simple enum for custom emote ids or emoji symbols
///
/// When written to URLs in the API (or `Display`ed), emojis become percent encoded, and custom emote ids
/// are prefixed with a colon (`:`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
pub struct ReactionShorthand {
    #[serde(flatten)]
    pub emote: EmoteOrEmoji,

    pub me: bool,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ReactionFull {
    pub emote: EmoteOrEmoji,
    pub users: Vec<Snowflake>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum Reaction {
    Shorthand(ReactionShorthand),
    Full(ReactionFull),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Attachment {
    #[serde(flatten)]
    pub file: File,
}
