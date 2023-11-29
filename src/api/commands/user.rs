use super::*;

command! {
    -struct UserRegister -> Session: POST[1000 ms, 1]("user") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct UserRegisterForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub email: SmolStr,
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub username: SmolStr,
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub password: SmolStr,

            #[serde(flatten)]
            pub dob: DateOfBirth,

            /// hCaptcha token
            pub token: String, // TODO: Don't allocate?
        }
    }

    -struct UserLogin -> Session: POST[1000 ms, 1]("user" / "@me") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct UserLoginForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub email: SmolStr,
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub password: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct Enable2FA -> Added2FA: POST[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct Enable2FAForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub password: SmolStr,
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub token: String,
        }
    }

    +struct Confirm2FA -> (): PATCH[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct Confirm2FAForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub password: SmolStr,
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub totp: SmolStr,
        }
    }

    +struct Remove2FA -> (): DELETE[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct Remove2FAForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub password: SmolStr,
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub totp: SmolStr,
        }
    }

    +struct ChangePassword -> (): PATCH[2000 ms, 1]("user" / "@me" / "password") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct ChangePasswordForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub current: SmolStr,

            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub new: SmolStr,

            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct GetSessions -> Vec<AnonymousSession>: GET[500 ms, 1]("user" / "@me" / "sessions") {}

    /// Clears all **other** sessions
    +struct ClearSessions -> (): DELETE[5000 ms, 1]("user" / "@me" / "sessions") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct ClearSessionsForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct GetRelationships -> Vec<Relationship>: GET("user" / "@me" / "relationships") {}

    +struct PatchRelationship -> Relationship: PATCH[1000 ms, 1]("user" / "@me" / "relationships" / user_id) {
        pub user_id: Snowflake,

        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        #[derive(Default)] struct PatchRelationshipBody {
            /// Your desired relationship with the other user
            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub rel: Nullable<UserRelationship>,

            /// Optional note to give the user
            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub note: Nullable<SmolStr>,
        }
    }

    +struct UpdateUserProfile -> UserProfile: PATCH[500 ms, 1]("user" / "@me" / "profile") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        #[derive(Default)] struct UpdateUserProfileBody {
            pub bits: UserProfileBits,

            #[serde(default, skip_serializing_if = "ExtraUserProfileBits::is_empty")]
            #[cfg_attr(feature = "builder", builder(default))]
            pub extra: ExtraUserProfileBits,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub nick: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub avatar: Nullable<Snowflake>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub banner: Nullable<Snowflake>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub status: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub bio: Nullable<SmolStr>,
        }
    }

    /// Fetches full user information, including profile data
    +struct GetUser -> User: GET("user" / user_id) {
        pub user_id: Snowflake,
    }

    +struct UpdateUserPrefs -> (): PATCH[200 ms]("user" / "@me" / "prefs") {
        ; struct UpdateUserPrefsBody {
            #[serde(flatten)]
            pub inner: UserPreferences,
        }
    }
}

impl From<UserPreferences> for UpdateUserPrefsBody {
    fn from(inner: UserPreferences) -> UpdateUserPrefsBody {
        UpdateUserPrefsBody { inner }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct Added2FA {
    pub url: String,
    pub backup: Vec<String>,
}
