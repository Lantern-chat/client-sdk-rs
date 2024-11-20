use super::*;

command! { User;
    -struct UserRegister(U) -> One Session: POST[1000 ms, 1]("user") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct UserRegisterForm {
            /// Email address
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub email: SmolStr,

            /// Username
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub username: SmolStr,

            /// Password (Plaintext, will be hashed on the server)
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub password: SmolStr,

            /// Date of birth
            pub dob: Timestamp,

            /// hCaptcha token
            pub token: String, // TODO: Don't allocate?
        }
    }

    -struct UserLogin(U) -> One Session: POST[1000 ms, 1]("user" / "@me") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct UserLoginForm {
            /// Email address
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub email: SmolStr,

            /// Password (Plaintext, will be hashed on the server)
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub password: SmolStr,

            /// 2FA token, if enabled
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct UserLogout(U) -> One (): DELETE[1000 ms, 1]("user" / "@me") {}

    +struct Enable2FA(U) -> One Added2FA: POST[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct Enable2FAForm {
            /// Password
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub password: SmolStr,

            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub token: String,
        }
    }

    +struct Confirm2FA(U) -> One (): PATCH[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct Confirm2FAForm {
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub password: SmolStr,

            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub totp: SmolStr,
        }
    }

    +struct Remove2FA(U) -> One (): DELETE[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct Remove2FAForm {
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub password: SmolStr,

            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub totp: SmolStr,
        }
    }

    +struct ChangePassword(U) -> One (): PATCH[2000 ms, 1]("user" / "@me" / "password") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct ChangePasswordForm {
            /// Current password
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub current: SmolStr,

            /// New password
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub new: SmolStr,

            /// 2FA token, if enabled
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct GetSessions(U) -> Many AnonymousSession: GET[500 ms, 1]("user" / "@me" / "sessions") {}

    /// Clears all **other** sessions
    +struct ClearSessions(U) -> One (): DELETE[5000 ms, 1]("user" / "@me" / "sessions") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        struct ClearSessionsForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct GetRelationships(U) -> Many Relationship: GET("user" / "@me" / "relationships") {}

    +struct PatchRelationship(U) -> One Relationship: PATCH[1000 ms, 1]("user" / "@me" / "relationships" / user_id) {
        pub user_id: UserId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        #[derive(Default)] struct PatchRelationshipBody {
            /// Your desired relationship with the other user
            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub rel: Nullable<UserRelationship>,

            /// Optional note to give the user
            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(into))]
            pub note: Nullable<SmolStr>,
        }
    }

    +struct UpdateUserProfile -> One UserProfile: PATCH[500 ms, 1]("user" / "@me" / "profile") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", derive(bon::Builder))]
        #[derive(Default)] struct UpdateUserProfileBody {
            pub bits: UserProfileBits,

            #[serde(default, skip_serializing_if = "ExtraUserProfileBits::is_empty")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            #[cfg_attr(feature = "bon", builder(default))]
            pub extra: ExtraUserProfileBits,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default, into))]
            pub nick: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default, into))]
            pub avatar: Nullable<FileId>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default, into))]
            pub banner: Nullable<FileId>,

            #[serde(default, skip_serializing_if = "is_default")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            #[cfg_attr(feature = "bon", builder(default))]
            pub banner_align: BannerAlign,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default, into))]
            pub status: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            #[cfg_attr(feature = "bon", builder(default, into))]
            pub bio: Nullable<SmolStr>,
        }
    }

    /// Fetches full user information, including profile data
    +struct GetUser -> One User: GET("user" / user_id) {
        pub user_id: UserId,
    }

    +struct UpdateUserPrefs(U) -> One (): PATCH[200 ms]("user" / "@me" / "prefs") {
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

decl_enum! {
    #[derive(Default, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
    #[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "command"))]
    pub enum BannerAlign: u8 {
        #[default]
        0 = Top,
        1 = Middle,
        2 = Bottom,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef), ts(tag = "command"))]
pub struct Added2FA {
    /// URL to be displayed as a QR code and added to an authenticator app
    pub url: String,
    /// Backup codes to be stored in a safe place
    pub backup: Vec<String>,
}
