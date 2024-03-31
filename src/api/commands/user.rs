use super::*;

command! {
    -struct UserRegister -> One Session: POST[1000 ms, 1]("user") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct UserRegisterForm {
            /// Email address
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub email: SmolStr,

            /// Username
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub username: SmolStr,

            /// Password (Plaintext, will be hashed on the server)
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub password: SmolStr,

            /// Date of birth
            pub dob: Timestamp,

            /// hCaptcha token
            pub token: String, // TODO: Don't allocate?
        }
    }

    -struct UserLogin -> One Session: POST[1000 ms, 1]("user" / "@me") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct UserLoginForm {
            /// Email address
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub email: SmolStr,
            /// Password (Plaintext, will be hashed on the server)
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub password: SmolStr,

            /// 2FA token, if enabled
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct UserLogout -> One (): DELETE[1000 ms, 1]("user" / "@me") {}

    +struct Enable2FA -> One Added2FA: POST[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct Enable2FAForm {
            /// Password
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub password: SmolStr,

            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub token: String,
        }
    }

    +struct Confirm2FA -> One (): PATCH[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct Confirm2FAForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub password: SmolStr,
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub totp: SmolStr,
        }
    }

    +struct Remove2FA -> One (): DELETE[2000 ms, 1]("user" / "@me" / "2fa") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct Remove2FAForm {
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub password: SmolStr,
            #[cfg_attr(feature = "builder", builder(setter(into)))]
            pub totp: SmolStr,
        }
    }

    +struct ChangePassword -> One (): PATCH[2000 ms, 1]("user" / "@me" / "password") {
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

    +struct GetSessions -> Many AnonymousSession: GET[500 ms, 1]("user" / "@me" / "sessions") {}

    /// Clears all **other** sessions
    +struct ClearSessions -> One (): DELETE[5000 ms, 1]("user" / "@me" / "sessions") {
        ;
        #[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
        struct ClearSessionsForm {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub totp: Option<SmolStr>,
        }
    }

    +struct GetRelationships -> Many Relationship: GET("user" / "@me" / "relationships") {}

    +struct PatchRelationship -> One Relationship: PATCH[1000 ms, 1]("user" / "@me" / "relationships" / user_id) {
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

    +struct UpdateUserProfile -> One UserProfile: PATCH[500 ms, 1]("user" / "@me" / "profile") {
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

            #[serde(default, skip_serializing_if = "is_default")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub banner_align: BannerAlign,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub status: Nullable<SmolStr>,

            #[serde(default, skip_serializing_if = "Nullable::is_undefined")]
            #[cfg_attr(feature = "builder", builder(default, setter(into)))]
            pub bio: Nullable<SmolStr>,
        }
    }

    /// Fetches full user information, including profile data
    +struct GetUser -> One User: GET("user" / user_id) {
        pub user_id: Snowflake,
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[cfg_attr(feature = "rkyv", derive(rkyv::CheckBytes))]
#[repr(u8)]
pub enum BannerAlign {
    #[default]
    Top = 0,
    Middle = 1,
    Bottom = 2,
}

common::impl_rkyv_for_pod!(BannerAlign);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct Added2FA {
    pub url: String,
    pub backup: Vec<String>,
}
