use super::*;

command! {
    /// Create message command
    struct CreateMessage -> Message: POST("rooms" / room_id / "messages") where Room::SEND_MESSAGES {
        #[serde(skip)]
        pub room_id: Snowflake;

        struct CreateMessageBody {
            pub content: SmolStr,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub parent: Option<Snowflake>,

            #[serde(skip_serializing_if = "Vec::is_empty")]
            pub attachments: Vec<Snowflake> where Room::ATTACH_FILES if !attachments.is_empty(),
        }
    }

    struct GetMessage -> Message: GET("rooms" / room_id / "messages" / msg_id) where Room::READ_MESSAGES {
        room_id: Snowflake,
        msg_id: Snowflake,
    }

    struct GetMessages -> Vec<Message>: GET("rooms" / room_id / "messages" ? thread & after) where Room::READ_MESSAGES {
        room_id: Snowflake,
        after: Option<Snowflake>,
        thread: Option<Snowflake>,
    }

    struct StartTyping -> (): POST("rooms" / room_id / "typing") where Room::SEND_MESSAGES {
        #[serde(skip)]
        room_id: Snowflake,
    }
}
