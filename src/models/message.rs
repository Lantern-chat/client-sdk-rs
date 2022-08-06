use super::*;

bitflags::bitflags! {
    pub struct MessageFlags: i16 {
        const DELETED           = 1 << 0;
        const MENTIONS_EVERYONE = 1 << 1;
        const MENTIONS_HERE     = 1 << 2;
        const TTS               = 1 << 3;
        const SUPRESS_EMBEDS    = 1 << 4;

        /// Top 6 bits are a language code,
        /// which is never actually exposed to users.
        const LANGUAGE          = 0b111_111 << (16 - 6);

        const PRIVATE_FLAGS     = Self::DELETED.bits | Self::LANGUAGE.bits;
    }
}

impl_serde_for_bitflags!(MessageFlags - MessageFlags::PRIVATE_FLAGS);
impl_schema_for_bitflags!(MessageFlags);
impl_pg_for_bitflags!(MessageFlags);

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[derive(enum_primitive_derive::Primitive)]
#[repr(i16)]
pub enum MessageKind {
    Normal  = 0,
    Welcome = 1,
}

impl Default for MessageKind {
    fn default() -> MessageKind {
        MessageKind::Normal
    }
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

    /// Partial PartyMember
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<PartyMember>,

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ReactionShorthand {
    pub emote: Snowflake,
    pub own: bool,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ReactionFull {
    pub emote: Emote,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PinFolder {
    pub id: Snowflake,
    pub name: SmolStr,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_id: Option<Snowflake>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<SmolStr>,
}
