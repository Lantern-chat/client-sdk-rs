use super::*;

command! {
    /// Create message command
    +struct CreateMessage -> One Message: POST[100 ms, 2]("room" / room_id / "messages") where SEND_MESSAGES {
        pub room_id: RoomId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        struct CreateMessageBody {
            #[serde(default)]
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub content: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub parent: Option<RoomId>,

            #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub attachments: ThinVec<Snowflake> where ATTACH_FILES if !attachments.is_empty(),

            #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub embeds: ThinVec<Embed> where EMBED_LINKS if !embeds.is_empty(),

            #[serde(default, skip_serializing_if = "is_false")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub ephemeral: bool,

            #[serde(default, skip_serializing_if = "is_false")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub tts: bool,
        }
    }

    +struct EditMessage -> One Message: PATCH[500 ms, 2]("room" / room_id / "messages" / msg_id) where SEND_MESSAGES {
        pub room_id: RoomId,
        pub msg_id: MessageId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        struct EditMessageBody {
            #[serde(default)]
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub content: SmolStr,

            #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub attachments: ThinVec<Snowflake>,
        }
    }

    +struct DeleteMessage -> One (): DELETE("room" / room_id / "messages" / msg_id) where READ_MESSAGE_HISTORY {
        pub room_id: RoomId,
        pub msg_id: MessageId,
    }

    +struct GetMessage -> One Message: GET("room" / room_id / "messages" / msg_id) where READ_MESSAGE_HISTORY {
        pub room_id: RoomId,
        pub msg_id: MessageId,
    }

    +struct StartTyping -> One (): POST[100 ms]("room" / room_id / "typing") where SEND_MESSAGES {
        pub room_id: RoomId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        #[derive(Default)] struct StartTypingBody {
            /// Will only show within the parent context if set
            #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub parent: Option<MessageId>,
        }
    }

    +struct GetMessages -> Many Message: GET("room" / room_id / "messages") where READ_MESSAGE_HISTORY {
        pub room_id: RoomId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        #[derive(Default)] struct GetMessagesQuery {
            #[serde(flatten, default, alias = "q", skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub query: Option<Cursor>,

            #[serde(default, alias = "thread", skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub parent: Option<MessageId>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub limit: Option<u8>,

            #[serde(default, alias = "pins", skip_serializing_if = "ThinVec::is_empty")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub pinned: ThinVec<FolderId>,

            /// If true, return only messages in the channel which have been starred by us
            #[serde(default, skip_serializing_if = "crate::models::is_false")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub starred: bool,

            /// If above zero, this will also fetch child messages of messages
            ///
            /// Max level is 5
            ///
            /// Child messages will not obey other filtering criteria.
            #[serde(default, alias = "depth", skip_serializing_if = "crate::models::is_default")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub recurse: u8,
        }
    }

    +struct PinMessage -> One (): PUT("room" / room_id / "messages" / msg_id / "pins" / pin_tag) {
        pub room_id: RoomId,
        pub msg_id: MessageId,
        pub pin_tag: FolderId,
    }

    +struct UnpinMessage -> One (): DELETE("room" / room_id / "messages" / msg_id / "pins" / pin_tag) {
        pub room_id: RoomId,
        pub msg_id: MessageId,
        pub pin_tag: FolderId,
    }

    +struct StarMessage -> One (): PUT("room" / room_id / "messages" / msg_id / "star") {
        pub room_id: RoomId,
        pub msg_id: MessageId,
    }

    +struct UnstarMessage -> One (): DELETE("room" / room_id / "messages" / msg_id / "star") {
        pub room_id: RoomId,
        pub msg_id: MessageId,
    }

    +struct PutReaction -> One (): PUT("room" / room_id / "messages" / msg_id / "reactions" / emote_id / "@me") {
        pub room_id: RoomId,
        pub msg_id: MessageId,
        pub emote_id: EmoteOrEmoji,
    }

    +struct DeleteOwnReaction -> One (): DELETE("room" / room_id / "messages" / msg_id / "reactions" / emote_id / "@me") {
        pub room_id: RoomId,
        pub msg_id: MessageId,
        pub emote_id: EmoteOrEmoji,
    }

    +struct DeleteUserReaction -> One (): DELETE("room" / room_id / "messages" / msg_id / "reactions" / emote_id / user_id) {
        pub room_id: RoomId,
        pub msg_id: MessageId,
        pub emote_id: EmoteOrEmoji,
        pub user_id: UserId,
    }

    +struct DeleteAllReactions -> One (): DELETE("room" / room_id / "messages" / msg_id / "reactions") {
        pub room_id: RoomId,
        pub msg_id: MessageId,
    }

    // TODO
    +struct GetReactions -> Many (): GET("room" / room_id / "messages" / msg_id / "reactions" / emote_id) {
        pub room_id: RoomId,
        pub msg_id: MessageId,
        pub emote_id: EmoteOrEmoji,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        struct GetReactionsForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            after: Option<Snowflake>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            limit: Option<i8>,
        }
    }

    +struct GetRoom -> One FullRoom: GET("room" / room_id) {
        pub room_id: RoomId,
    }

    +struct PatchRoom -> One FullRoom: PATCH[500 ms, 1]("room" / room_id) {
        pub room_id: RoomId,

        ;
        /// `Nullable::Undefined` or `Option::None` fields indicate no change
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        #[cfg_attr(feature = "rkyv", archive(compare(PartialEq)))] // waiting on https://github.com/rkyv/rkyv/pull/525
        #[derive(Default, PartialEq)]
        struct PatchRoomForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub name: Option<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub topic: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub avatar: Nullable<Snowflake>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub position: Option<u8>,

            #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub remove_overwrites: ThinVec<Snowflake>,

            #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub overwrites: ThinVec<Overwrite>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub nsfw: Option<bool>,
        }
    }

    +struct DeleteRoom -> One (): DELETE[500 ms, 1]("room" / room_id) {
        pub room_id: RoomId,
    }
}
