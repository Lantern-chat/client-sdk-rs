use super::*;

command! {
    +struct GetParty -> Party: GET("party" / party_id) {
        pub party_id: Snowflake,
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
        #[derive(Default)] struct CreatePinFolderForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub name: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub description: Option<SmolStr>,
        }
    }
}
