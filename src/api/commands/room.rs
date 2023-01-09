use super::*;

command! {
    /// Create message command
    +struct CreateMessage -> Message: POST("room" / room_id / "messages") where Room::SEND_MESSAGES {
        pub room_id: Snowflake,

        ; struct CreateMessageBody {
            #[serde(default)]
            pub content: SmolStr,

            //#[serde(default, skip_serializing_if = "is_false")]
            //pub tts: bool,

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

    +struct GetMessages -> Vec<Message>: GET("room" / room_id / "messages") where Room::READ_MESSAGE_HISTORY {
        pub room_id: Snowflake,

        ; #[derive(Default)] struct GetMessagesQuery {
            #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
            pub query: Option<Cursor>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub thread: Option<Snowflake>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub limit: Option<u8>,
        }
    }

    +struct PutReaction -> (): PUT("room" / room_id / "messages" / msg_id / "reactions" / emote_id / "@me") {
        pub room_id: Snowflake,
        pub msg_id: Snowflake,
        pub emote_id: EmoteOrEmoji,
    }

    +struct DeleteOwnReaction -> (): DELETE("room" / room_id / "messages" / msg_id / "reactions" / emote_id / "@me") {
        pub room_id: Snowflake,
        pub msg_id: Snowflake,
        pub emote_id: EmoteOrEmoji,
    }

    +struct DeleteUserReaction -> (): DELETE("room" / room_id / "messages" / msg_id / "reactions" / emote_id / user_id) {
        pub room_id: Snowflake,
        pub msg_id: Snowflake,
        pub emote_id: EmoteOrEmoji,
        pub user_id: Snowflake,
    }

    +struct DeleteAllReactions -> (): DELETE("room" / room_id / "messages" / msg_id / "reactions") {
        pub room_id: Snowflake,
        pub msg_id: Snowflake,
    }

    +struct GetReactions -> Vec<()>: GET("room" / room_id / "messages" / msg_id / "reactions" / emote_id) {
        pub room_id: Snowflake,
        pub msg_id: Snowflake,
        pub emote_id: EmoteOrEmoji,

        ; struct GetReactionsForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            after: Option<Snowflake>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            limit: Option<i8>,
        }
    }
}
