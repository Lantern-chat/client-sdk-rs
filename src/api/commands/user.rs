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

    +struct SetUserAvatar -> (): POST("user" / "@me" / "avatar" / file_id) {
        pub file_id: Snowflake,
    }

    +struct DeleteUserAvatar -> (): DELETE("user" / "@me" / "avatar") {}

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
