use super::*;

command! {
    /// Create message command
    +struct CreateMessage -> Message: POST("room" / room_id / "messages") where Room::SEND_MESSAGES {
        pub room_id: Snowflake,

        ; struct CreateMessageBody {
            #[serde(default)]
            pub content: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub parent: Option<Snowflake>,

            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            pub attachments: Vec<Snowflake> where Room::ATTACH_FILES if !attachments.is_empty(),
        }
    }

    +struct EditMessage -> Message: PATCH("room" / room_id / "messages" / msg_id) where Room::SEND_MESSAGES {
        pub room_id: Snowflake,
        pub msg_id: Snowflake,

        ; struct EditMessageBody {
            #[serde(default)]
            pub content: SmolStr,

            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            pub attachments: Vec<Snowflake>,
        }
    }

    +struct GetMessage -> Message: GET("room" / room_id / "messages" / msg_id) where Room::READ_MESSAGE_HISTORY {
        pub room_id: Snowflake,
        pub msg_id: Snowflake,
    }

    +struct StartTyping -> (): POST("room" / room_id / "typing") where Room::SEND_MESSAGES {
        pub room_id: Snowflake,
    }
}

/// Directional search query
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum MessageSearch {
    After(Snowflake),
    Before(Snowflake),
}

command! {
    +struct GetMessages -> Vec<Message>: GET("room" / room_id / "messages") where Room::READ_MESSAGE_HISTORY {
        pub room_id: Snowflake,

        ; #[derive(Default)] struct GetMessagesQuery {
            #[serde(flatten)]
            pub query: Option<MessageSearch>,

            #[serde(default)]
            pub thread: Option<Snowflake>,

            #[serde(default)]
            pub limit: Option<u8>,
        }
    }
}
