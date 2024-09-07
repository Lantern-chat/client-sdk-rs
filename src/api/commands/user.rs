use super::*;

command! {
    -struct UserRegister -> One Session: POST[1000 ms, 1]("user") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        struct UserRegisterForm {
            /// Email address
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub email: SmolStr,

            /// Username
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub username: SmolStr,

            /// Password (Plaintext, will be hashed on the server)
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub password: SmolStr,

            /// Date of birth
            pub dob: Timestamp,

            /// hCaptcha token
            pub token: String, // TODO: Don't allocate?
        }
    }

    -struct UserLogin -> One Session: POST[1000 ms, 1]("user" / "@me") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        struct UserLoginForm {
            /// Email address
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub email: SmolStr,
            /// Password (Plaintext, will be hashed on the server)
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub password: SmolStr,

            /// 2FA token, if enabled
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct UserLogout -> One (): DELETE[1000 ms, 1]("user" / "@me") {}

    +struct Enable2FA -> One Added2FA: POST[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        struct Enable2FAForm {
            /// Password
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub password: SmolStr,

            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub token: String,
        }
    }

    +struct Confirm2FA -> One (): PATCH[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        struct Confirm2FAForm {
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub password: SmolStr,
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub totp: SmolStr,
        }
    }

    +struct Remove2FA -> One (): DELETE[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        struct Remove2FAForm {
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub password: SmolStr,
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub totp: SmolStr,
        }
    }

    +struct ChangePassword -> One (): PATCH[2000 ms, 1]("user" / "@me" / "password") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        struct ChangePasswordForm {
            /// Current password
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub current: SmolStr,

            /// New password
            #[cfg_attr(feature = "typed-builder", builder(setter(into)))]
            pub new: SmolStr,

            /// 2FA token, if enabled
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct GetSessions -> Many AnonymousSession: GET[500 ms, 1]("user" / "@me" / "sessions") {}

    /// Clears all **other** sessions
    +struct ClearSessions -> One (): DELETE[5000 ms, 1]("user" / "@me" / "sessions") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        struct ClearSessionsForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct GetRelationships -> Many Relationship: GET("user" / "@me" / "relationships") {}

    +struct PatchRelationship -> One Relationship: PATCH[1000 ms, 1]("user" / "@me" / "relationships" / user_id) {
        pub user_id: UserId,

        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        #[derive(Default)] struct PatchRelationshipBody {
            /// Your desired relationship with the other user
            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub rel: Nullable<UserRelationship>,

            /// Optional note to give the user
            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub note: Nullable<SmolStr>,
        }
    }

    +struct UpdateUserProfile -> One UserProfile: PATCH[500 ms, 1]("user" / "@me" / "profile") {
        ;
        #[cfg_attr(feature = "typed-builder", derive(typed_builder::TypedBuilder))]
        #[cfg_attr(feature = "bon", bon::builder)]
        #[derive(Default)] struct UpdateUserProfileBody {
            pub bits: UserProfileBits,

            #[serde(default, skip_serializing_if = "ExtraUserProfileBits::is_empty")]
            #[cfg_attr(feature = "typed-builder", builder(default))]
            pub extra: ExtraUserProfileBits,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub nick: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub avatar: Nullable<FileId>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub banner: Nullable<FileId>,

            #[serde(default, skip_serializing_if = "is_default")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub banner_align: BannerAlign,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub status: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "typed-builder", builder(default, setter(into)))]
            pub bio: Nullable<SmolStr>,
        }
    }

    /// Fetches full user information, including profile data
    +struct GetUser -> One User: GET("user" / user_id) {
        pub user_id: UserId,
    }

    +struct UpdateUserPrefs -> One (): PATCH[200 ms]("user" / "@me" / "prefs") {
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
    pub enum BannerAlign: u8 {
        #[default]
        0 = Top,
        1 = Middle,
        2 = Bottom,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize),
    archive(check_bytes)
)]
pub struct Added2FA {
    /// URL to be display as a QR code and added to an authenticator app
    pub url: String,
    /// Backup codes to be stored in a safe place
    pub backup: Vec<String>,
}
