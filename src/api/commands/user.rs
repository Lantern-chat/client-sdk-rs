use super::*;

command! {
    -struct UserRegister -> Session: POST("user") {
        ; struct UserRegisterForm {
            pub email: SmolStr,
            pub username: SmolStr,
            pub password: SmolStr,

            #[serde(flatten)]
            pub dob: DateOfBirth,

            /// hCaptcha token
            pub token: String, // TODO: Don't allocate?
        }
    }

    -struct UserLogin -> Session: POST("user" / "@me") {
        ; struct UserLoginForm {
            pub email: SmolStr,
            pub password: SmolStr,

            #[serde(default)]
            pub totp: Option<SmolStr>,
        }
    }

    +struct GetSessions -> Vec<AnonymousSession>: GET("user" / "@me" / "sessions") {}

    +struct GetFriends -> Vec<Friend>: GET("user" / "@me" / "friends") {}

    /// Used for sending and accepting friend requests
    +struct AddFriend -> Friend: POST("user" / "@me" / "friends" / user_id) {
        pub user_id: Snowflake,
    }

    /// Used for rejecting a friend-request or removing an existing friend
    +struct RemoveFriend -> (): DELETE("user" / "@me" / "friends" / user_id) {
        pub user_id: Snowflake,
    }

    +struct PatchFriend -> Friend: PATCH("user" / "@me" / "friends" / user_id) {
        pub user_id: Snowflake,

        ; #[derive(Default)] struct PatchFriendBody {
            pub fav: Nullable<bool>,
            pub note: Nullable<SmolStr>,
        }
    }

    +struct UpdateUserProfile -> UserProfile: PATCH("user" / "@me" / "profile") {
        ; #[derive(Default)] struct UpdateUserProfileBody {
            pub bits: UserProfileBits,

            #[serde(default, skip_serializing_if = "ExtraUserProfileBits::is_empty")]
            pub extra: ExtraUserProfileBits,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            pub nick: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            pub avatar: Nullable<Snowflake>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            pub banner: Nullable<Snowflake>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            pub status: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            pub bio: Nullable<SmolStr>,
        }
    }

    +struct GetUserProfile -> UserProfile: GET("user" / user_id / "profile") {
        pub user_id: Snowflake,
    }

    +struct GetUser -> User: GET("user" / user_id) {
        pub user_id: Snowflake,
    }

    +struct UpdateUserPrefs -> (): PATCH("user" / "@me" / "prefs") {
        ; struct UpdateUserPrefsBody {
            #[serde(flatten)]
            prefs: UserPreferences,
        }
    }
}

impl From<UserPreferences> for UpdateUserPrefsBody {
    fn from(prefs: UserPreferences) -> UpdateUserPrefsBody {
        UpdateUserPrefsBody { prefs }
    }
}
