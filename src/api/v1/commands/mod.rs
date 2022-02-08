use smol_str::SmolStr;

use crate::models::*;

command! {
    /// Create message command
    struct CreateMessage -> Message: POST("rooms" / room_id / "messages") where Room::SEND_MESSAGES {
        #[serde(skip)]
        pub room_id: Snowflake,

        pub content: SmolStr,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub parent: Option<Snowflake>,

        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub attachments: Vec<Snowflake> where Room::ATTACH_FILES if |a: &[_]| !a.is_empty(),
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
}
