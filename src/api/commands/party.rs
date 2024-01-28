use super::*;

command! {
    +struct GetParty -> One Party: GET("party" / party_id) {
        pub party_id: Snowflake,
    }

    +struct CreateParty -> One Party: POST[5000 ms, 1]("party") {
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

    +struct PatchParty -> One Party: PATCH[500 ms, 1]("party" / party_id) {
        pub party_id: Snowflake,

        ;
        #[derive(Default, PartialEq)]
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "rkyv", archive(compare(PartialEq)))]
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

    +struct DeleteParty -> One (): DELETE("party" / party_id) {
        pub party_id: Snowflake,
    }

    // TODO: Use same command for accepting?
    +struct TransferOwnership -> One (): PUT("party" / party_id / "owner" / user_id) {
        pub party_id: Snowflake,
        pub user_id: Snowflake,
    }

    +struct CreateRole -> One Role: POST[1000 ms, 1]("party" / party_id / "roles") {
        pub party_id: Snowflake,

        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct CreateRoleForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub name: SmolStr,
        }
    }

    +struct PatchRole -> One Role: PATCH("party" / party_id / "roles" / role_id) {
        pub party_id: Snowflake,
        pub role_id: Snowflake,

        ;
        #[derive(Default, PartialEq)]
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "rkyv", archive(compare(PartialEq)))]
        struct PatchRoleForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub flags: Option<RoleFlags>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub name: Option<SmolStr>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub color: Option<u32>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub permissions: Option<Permissions>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub avatar: Nullable<Snowflake>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub position: Option<u8>,
        }
    }

    +struct DeleteRole -> One (): DELETE("party" / party_id / "roles" / role_id) {
        pub party_id: Snowflake,
        pub role_id: Snowflake,
    }

    +struct GetPartyMembers -> Many PartyMember: GET("party" / party_id / "members") {
        pub party_id: Snowflake,
    }

    +struct GetPartyMember -> One PartyMember: GET("party" / party_id / "member" / member_id) {
        pub party_id: Snowflake,
        pub member_id: Snowflake,
    }

    +struct GetPartyRooms -> Many Room: GET("party" / party_id / "rooms") {
        pub party_id: Snowflake,
    }

    +struct GetPartyInvites -> Many Invite: GET("party" / party_id / "invites") {
        pub party_id: Snowflake,
    }

    +struct GetMemberProfile -> One UserProfile: GET("party" / party_id / "members" / user_id / "profile") {
        pub party_id: Snowflake,
        pub user_id: Snowflake,
    }

    +struct UpdateMemberProfile -> One UserProfile: PATCH("party" / party_id / "members" / "profile") {
        pub party_id: Snowflake,

        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct UpdateMemberProfileBody {
            #[serde(flatten)]
            pub profile: user::UpdateUserProfileBody,
        }
    }

    +struct CreatePartyInvite -> One Invite: POST[2000 ms, 1]("party" / party_id / "invites") {
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

    +struct CreatePinFolder -> One PinFolder: POST("party" / party_id / "pins") {
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

    +struct CreateRoom -> One Room: POST[5000 ms, 1]("party" / party_id / "rooms") {
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
            #[cfg_attr(feature = "rkyv", with(rkyv::with::CopyOptimize))]
            pub overwrites: ThinVec<Overwrite>,

            #[serde(default)]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub position: i16,
        }
    }

    +struct SearchParty -> One (): POST("party" / party_id / "search") {
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
#[cfg_attr(feature = "rkyv", derive(rkyv::CheckBytes))]
#[repr(u8)]
pub enum CreateRoomKind {
    #[default]
    Text = RoomKind::Text as u8,
    Voice = RoomKind::Voice as u8,
    UserForum = RoomKind::UserForum as u8,
}

common::impl_rkyv_for_pod!(CreateRoomKind);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct PartySettings {
    pub flags: PartyFlags,
    pub prefs: PartyPreferences,
}
