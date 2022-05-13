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

    +struct CreatePartyInvite -> Invite: POST("party" / party_id / "invites") {
        pub party_id: Snowflake,

        ;
        /// Infinite parameters may only be used with appropriate permissions
        struct CreatePartyInviteBody {
            /// If `None`, invite has infinite uses
            #[serde(default)]
            pub max_uses: Option<u16>,

            /// If `None`, invite has infinite duration
            #[serde(default)]
            pub duration: Option<u64>,

            #[serde(default)]
            pub description: Option<SmolStr>,
        }
    }
}
