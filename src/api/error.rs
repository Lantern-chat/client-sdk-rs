//! Standard API error types and codes.

use std::{borrow::Cow, fmt};

use http::StatusCode;

/// Standard API error response, containing an error code and message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct ApiError {
    /// Error code
    pub code: ApiErrorCode,

    /// Human-readable error message
    #[cfg_attr(feature = "rkyv", with(rkyv::with::AsOwned))]
    pub message: Cow<'static, str>,
}

#[cfg(feature = "rkyv")]
impl ArchivedApiError {
    /// Get the error code for this error.
    pub fn code(&self) -> ApiErrorCode {
        rkyv::Deserialize::deserialize(&self.code, &mut rkyv::Infallible)
            .unwrap_or_else(|_| unsafe { core::hint::unreachable_unchecked() })
    }
}

impl fmt::Debug for ArchivedApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchivedApiError").field("code", &self.code()).field("message", &self.message).finish()
    }
}

macro_rules! error_codes {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $repr:ty $(= $unknown:ident)? {$(
            $(#[$variant_meta:meta])*
            $variant:ident = $code:literal = $status:expr,
        )*}
    ) => {
        common::enum_codes! {
            $(#[$meta])*
            $vis enum $name: $repr $(= $unknown)? {$(
                $(#[$variant_meta])*
                $variant = $code,
            )*}
        }

        impl $name {
            /// Get the HTTP status code for this error code.
            pub fn http_status(self) -> StatusCode {
                match self {
                    $(Self::$variant => $status,)*
                }
            }
        }
    };
}

error_codes! {
    /// Standard API error codes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema_repr))]
    #[derive(enum_primitive_derive::Primitive)]
    pub enum ApiErrorCode: u16 = Unknown {
        // Server errors
        DbError                  = 50001 = StatusCode::INTERNAL_SERVER_ERROR,
        JoinError                = 50002 = StatusCode::INTERNAL_SERVER_ERROR,
        SemaphoreError           = 50003 = StatusCode::INTERNAL_SERVER_ERROR,
        HashError                = 50004 = StatusCode::INTERNAL_SERVER_ERROR,
        JsonError                = 50005 = StatusCode::INTERNAL_SERVER_ERROR,
        EventEncodingError       = 50006 = StatusCode::INTERNAL_SERVER_ERROR,
        InternalError            = 50007 = StatusCode::INTERNAL_SERVER_ERROR,
        Utf8ParseError           = 50008 = StatusCode::INTERNAL_SERVER_ERROR,
        IOError                  = 50009 = StatusCode::INTERNAL_SERVER_ERROR,
        InvalidHeaderValue       = 50010 = StatusCode::INTERNAL_SERVER_ERROR,
        XMLError                 = 50011 = StatusCode::INTERNAL_SERVER_ERROR,
        RequestError             = 50012 = StatusCode::INTERNAL_SERVER_ERROR,
        Unimplemented            = 50013 = StatusCode::INTERNAL_SERVER_ERROR,
        BincodeError             = 50014 = StatusCode::INTERNAL_SERVER_ERROR,
        CborError                = 50015 = StatusCode::INTERNAL_SERVER_ERROR,
        RkyvEncodingError        = 50016 = StatusCode::INTERNAL_SERVER_ERROR,

        // Client errors
        AlreadyExists            = 40001 = StatusCode::CONFLICT,
        UsernameUnavailable      = 40002 = StatusCode::BAD_REQUEST,
        InvalidEmail             = 40003 = StatusCode::UNAUTHORIZED,
        InvalidUsername          = 40004 = StatusCode::UNAUTHORIZED,
        InvalidPassword          = 40005 = StatusCode::UNAUTHORIZED,
        InvalidCredentials       = 40006 = StatusCode::UNAUTHORIZED,
        InsufficientAge          = 40007 = StatusCode::BAD_REQUEST,
        InvalidDate              = 40008 = StatusCode::BAD_REQUEST,
        InvalidContent           = 40009 = StatusCode::BAD_REQUEST,
        InvalidName              = 40010 = StatusCode::BAD_REQUEST,
        InvalidTopic             = 40011 = StatusCode::BAD_REQUEST,
        MissingUploadMetadataHeader  = 40012 = StatusCode::BAD_REQUEST,
        MissingAuthorizationHeader   = 40013 = StatusCode::BAD_REQUEST,
        NoSession                = 40014 = StatusCode::UNAUTHORIZED,
        InvalidAuthFormat        = 40015 = StatusCode::BAD_REQUEST,
        HeaderParseError         = 40016 = StatusCode::UNPROCESSABLE_ENTITY,
        MissingFilename          = 40017 = StatusCode::BAD_REQUEST,
        MissingMime              = 40018 = StatusCode::BAD_REQUEST,
        AuthTokenError           = 40019 = StatusCode::UNPROCESSABLE_ENTITY,
        Base64DecodeError        = 40020 = StatusCode::BAD_REQUEST,
        BodyDeserializeError     = 40021 = StatusCode::UNPROCESSABLE_ENTITY,
        QueryParseError          = 40022 = StatusCode::BAD_REQUEST,
        UploadError              = 40023 = StatusCode::BAD_REQUEST,
        InvalidPreview           = 40024 = StatusCode::BAD_REQUEST,
        MimeParseError           = 40025 = StatusCode::BAD_REQUEST,
        InvalidImageFormat       = 40026 = StatusCode::BAD_REQUEST,
        TOTPRequired             = 40027 = StatusCode::UNAUTHORIZED,
        InvalidPreferences       = 40028 = StatusCode::BAD_REQUEST,
        TemporarilyDisabled      = 40029 = StatusCode::FORBIDDEN,
        InvalidCaptcha           = 40030 = StatusCode::UNAUTHORIZED,
        Base85DecodeError        = 40031 = StatusCode::BAD_REQUEST,
        WebsocketError           = 40032 = StatusCode::BAD_REQUEST,
        MissingContentTypeHeader = 40033 = StatusCode::BAD_REQUEST,
        Blocked                  = 40034 = StatusCode::FORBIDDEN,
        Banned                   = 40035 = StatusCode::FORBIDDEN,
        SearchError              = 40036 = StatusCode::BAD_REQUEST,

        // Generic HTTP-like error codes
        BadRequest               = 40400 = StatusCode::BAD_REQUEST,
        Unauthorized             = 40401 = StatusCode::UNAUTHORIZED,
        NotFound                 = 40404 = StatusCode::NOT_FOUND,
        MethodNotAllowed         = 40405 = StatusCode::METHOD_NOT_ALLOWED,
        Conflict                 = 40409 = StatusCode::CONFLICT,
        RequestEntityTooLarge    = 40413 = unsafe { StatusCode::from_u16(413).unwrap_unchecked() }, // 413 Request Entity Too Large
        UnsupportedMediaType     = 40415 = StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ChecksumMismatch         = 40460 = unsafe { StatusCode::from_u16(460).unwrap_unchecked() }, // 460 Checksum Mismatch

        #[serde(other)]
        Unknown = 1 = StatusCode::IM_A_TEAPOT,
    }
}
