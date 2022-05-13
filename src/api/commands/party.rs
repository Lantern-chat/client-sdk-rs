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
}
