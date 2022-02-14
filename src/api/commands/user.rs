use super::*;

command! {
    -struct UserRegister -> Session: POST("user") {
        ; struct UserRegisterForm {
            pub email: SmolStr,
            pub username: SmolStr,
            pub password: SmolStr,

            /// Birth year
            pub year: i32,
            /// Birth month
            pub month: u8,
            /// Birth day
            pub day: u8,

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

    +struct SetUserPrefs -> (): PATCH("user" / "@me" / "prefs") {
        ; struct SetUserPrefsBody {
            #[serde(flatten)]
            prefs: UserPreferences,
        }
    }
}

impl From<UserPreferences> for SetUserPrefsBody {
    fn from(prefs: UserPreferences) -> SetUserPrefsBody {
        SetUserPrefsBody { prefs }
    }
}
