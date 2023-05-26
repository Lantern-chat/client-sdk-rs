use super::*;

command! {
    +struct GetParty -> Party: GET("party" / party_id) {
        pub party_id: Snowflake,
    }

    +struct CreateParty -> Party: POST("party") {
        ;

        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct CreatePartyForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub name: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub description: Option<SmolStr>,

            #[serde(default)]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub flags: PartyFlags,
        }
    }

    +struct PatchParty -> Party: PATCH("party" / party_id) {
        pub party_id: Snowflake,

        ;
        #[derive(Default, PartialEq)]
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct PatchPartyForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub name: Option<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub description: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub flags: Option<PartyFlags>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub default_room: Option<Snowflake>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub avatar: Nullable<Snowflake>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub banner: Nullable<Snowflake>,
        }
    }

    // TODO: Use same command for accepting?
    +struct TransferOwnership -> (): PUT("party" / party_id / "owner" / user_id) {
        pub party_id: Snowflake,
        pub user_id: Snowflake,
    }

    +struct GetPartyMembers -> Vec<PartyMember>: GET("party" / party_id / "members") {
        pub party_id: Snowflake,
    }

    +struct GetPartyRooms -> Vec<Room>: GET("party" / party_id / "rooms") {
        pub party_id: Snowflake,
    }

    +struct GetPartyInvites -> Vec<Invite>: GET("party" / party_id / "invites") {
        pub party_id: Snowflake,
    }

    +struct GetMemberProfile -> UserProfile: GET("party" / party_id / "members" / user_id / "profile") {
        pub party_id: Snowflake,
        pub user_id: Snowflake,
    }

    +struct UpdateMemberProfile -> UserProfile: PATCH("party" / party_id / "members" / "profile") {
        pub party_id: Snowflake,

        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct UpdateMemberProfileBody {
            #[serde(flatten)]
            profile: user::UpdateUserProfileBody,
        }
    }

    +struct CreatePartyInvite -> Invite: POST("party" / party_id / "invites") {
        pub party_id: Snowflake,

        ;
        /// Infinite parameters may only be used with appropriate permissions
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct CreatePartyInviteBody {
            /// If `None`, invite has infinite uses
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default))]
            pub max_uses: Option<u16>,

            /// If `None`, invite has infinite duration
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default))]
            pub duration: Option<u64>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub description: Option<SmolStr>,
        }
    }

    +struct CreatePinFolder -> PinFolder: POST("party" / party_id / "pins") {
        pub party_id: Snowflake,

        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct CreatePinFolderForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub name: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub description: Option<SmolStr>,
        }
    }

    +struct CreateRoom -> Room: POST("party" / party_id / "rooms") {
        pub party_id: Snowflake,

        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct CreateRoomForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub name: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub topic: Option<SmolStr>,

            #[cfg_attr(feature = "builder", builder(default))]
            pub kind: CreateRoomKind,

            #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub overwrites: ThinVec<Overwrite>,

            #[serde(default)]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub position: i16,
        }
    }

    +struct SearchParty -> (): POST("party" / party_id / "search") {
        pub party_id: Snowflake,

        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct SearchQuery {
            #[serde(flatten)]
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub query: String,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum CreateRoomKind {
    #[default]
    Text = RoomKind::Text as u8,
    Voice = RoomKind::Voice as u8,
    UserForum = RoomKind::UserForum as u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
pub struct PartySettings {
    pub flags: PartyFlags,
    pub prefs: PartyPreferences,
}
