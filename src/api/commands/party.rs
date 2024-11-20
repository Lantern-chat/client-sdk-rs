use super::*;

command! { Party;

    +struct GetParty -> One Party: GET("party" / party_id) {
        pub party_id: PartyId,
    }

    +struct CreateParty -> One Party: POST[5000 ms, 1]("party") {
        ;

        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct CreatePartyForm {
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub name: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub description: Option<ThinString>,

            #[serde(default)]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            #[cfg_attr(feature = "bon", builder(default))]
            pub flags: PartyFlags,
        }
    }

    +struct PatchParty -> One Party: PATCH[500 ms, 1]("party" / party_id) {
        pub party_id: PartyId,

        ;
        #[derive(Default, PartialEq)]
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        #[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq)))]
        struct PatchPartyForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub name: Option<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default, into))]
            pub description: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub flags: Option<PartyFlags>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub default_room: Option<RoomId>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default, into))]
            pub avatar: Nullable<FileId>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default, into))]
            pub banner: Nullable<FileId>,

            #[serde(default, skip_serializing_if = "is_default")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub banner_align: all::BannerAlign,
        }
    }

    +struct DeleteParty(U) -> One (): DELETE("party" / party_id) {
        pub party_id: PartyId,
    }

    /// Transfer ownership of a party to another user.
    ///
    /// This command is only available to the current owner, or the person who is accepting ownership.
    ///
    /// Confirming ownership is done by the accepting user also sending a `TransferOwnership`
    /// command with the same parameters.
    +struct TransferOwnership(U) -> One (): PUT("party" / party_id / "owner" / user_id) {
        pub party_id: PartyId,
        pub user_id: UserId,
    }

    +struct CreateRole -> One Role: POST[1000 ms, 1]("party" / party_id / "roles") {
        pub party_id: PartyId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct CreateRoleForm {
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub name: SmolStr,
        }
    }

    +struct PatchRole -> One Role: PATCH("party" / party_id / "roles" / role_id) {
        pub party_id: PartyId,
        pub role_id: RoleId,

        ;
        #[derive(Default, PartialEq)]
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        #[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq)))]
        struct PatchRoleForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub flags: Option<RoleFlags>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub name: Option<SmolStr>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub color: Option<u32>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub permissions: Option<Permissions>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default, into))]
            pub avatar: Nullable<FileId>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub position: Option<u8>,
        }
    }

    +struct DeleteRole -> One (): DELETE("party" / party_id / "roles" / role_id) {
        pub party_id: PartyId,
        pub role_id: RoleId,
    }

    +struct GetPartyMembers -> Many PartyMember: GET("party" / party_id / "members") {
        pub party_id: PartyId,
    }

    +struct GetPartyMember -> One PartyMember: GET("party" / party_id / "member" / member_id) {
        pub party_id: PartyId,
        pub member_id: UserId,
    }

    +struct GetPartyRooms -> Many Room: GET("party" / party_id / "rooms") {
        pub party_id: PartyId,
    }

    +struct GetPartyInvites -> Many Invite: GET("party" / party_id / "invites") {
        pub party_id: PartyId,
    }

    +struct GetMemberProfile -> One UserProfile: GET("party" / party_id / "members" / user_id / "profile") {
        pub party_id: PartyId,
        pub user_id: UserId,
    }

    +struct UpdateMemberProfile -> One UserProfile: PATCH("party" / party_id / "members" / "profile") {
        pub party_id: PartyId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct UpdateMemberProfileBody {
            #[serde(flatten)]
            pub profile: user::UpdateUserProfileBody,
        }
    }

    +struct CreatePartyInvite -> One Invite: POST[2000 ms, 1]("party" / party_id / "invites") {
        pub party_id: PartyId,

        ;
        /// Infinite parameters may only be used with appropriate permissions
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct CreatePartyInviteBody {
            /// If `None`, invite has infinite uses
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub max_uses: Option<u16>,

            /// If `None`, invite has infinite duration
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub duration: Option<u64>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub description: Option<SmolStr>,
        }
    }

    +struct CreatePinFolder -> One PinFolder: POST("party" / party_id / "pins") {
        pub party_id: PartyId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct CreatePinFolderForm {
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub name: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub description: Option<SmolStr>,
        }
    }

    +struct CreateRoom -> One Room: POST[5000 ms, 1]("party" / party_id / "rooms") {
        pub party_id: PartyId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct CreateRoomForm {
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub name: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub topic: Option<SmolStr>,

            #[cfg_attr(feature = "typed-builder", builder(default))]
            #[cfg_attr(feature = "bon", builder(default))]
            pub kind: CreateRoomKind,

            #[serde(default, skip_serializing_if = "ThinVec::is_empty")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default, into))]
            pub overwrites: ThinVec<Overwrite>,

            #[serde(default)]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default))]
            pub position: i16,
        }
    }

    +struct SearchParty -> One (): POST("party" / party_id / "search") {
        pub party_id: PartyId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct SearchQuery {
            #[serde(alias = "q")]
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub query: ThinString,
        }
    }
}

decl_enum! {
    #[derive(Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "command"))]
    pub enum CreateRoomKind: u8 {
        #[default]
        0 = Text,
        3 = Voice,
        4 = UserForum,
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
#[cfg_attr(feature = "bon", derive(bon::Builder))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "command"))]
pub struct PartySettings {
    pub flags: PartyFlags,
    pub prefs: PartyPreferences,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_room_kind_equivalence() {
        assert_eq!(CreateRoomKind::Text as u8, RoomKind::Text as u8);
        assert_eq!(CreateRoomKind::Voice as u8, RoomKind::Voice as u8);
        assert_eq!(CreateRoomKind::UserForum as u8, RoomKind::UserForum as u8);
    }
}
