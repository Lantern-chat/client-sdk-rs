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

    +struct PatchUserProfile -> UserProfile: PATCH("user" / "@me" / "profile") {
        ;

        struct PartialUserProfile {
            pub bits: UserProfileBits,

            /// File ID
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub avatar: Option<Snowflake>,

            /// File ID
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub banner: Option<Snowflake>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub status: Option<SmolStr>,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub bio: Option<SmolStr>,
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