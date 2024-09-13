//! Standard API error types and codes.

use std::{borrow::Cow, fmt};

use http::StatusCode;

/// Standard API error response, containing an error code and message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
pub struct ApiError {
    /// Error code
    pub code: ApiErrorCode,

    /// Human-readable error message
    #[cfg_attr(feature = "rkyv", rkyv(with = rkyv::with::AsOwned))]
    pub message: Cow<'static, str>,
}

#[cfg(feature = "rkyv")]
impl ArchivedApiError {
    /// Get the error code for this error.
    #[inline]
    #[must_use]
    pub const fn code(&self) -> ApiErrorCode {
        self.code.get()
    }
}

impl fmt::Debug for ArchivedApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchivedApiError").field("code", &self.code()).field("message", &self.message).finish()
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code as u16, self.message)
    }
}

impl core::error::Error for ApiError {}

macro_rules! error_codes {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $repr:ty $(= $unknown:ident)? {$(
            $(#[$variant_meta:meta])*
            $code:literal = $variant:ident = $status:expr,
        )*}
    ) => {
        enum_codes! {
            $(#[$meta])*
            $vis enum $name: pub $repr $(= $unknown)? {$(
                $(#[$variant_meta])*
                $code = $variant,
            )*}
        }

        impl $name {
            /// Get the HTTP status code for this error code.
            #[must_use]
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
    #[cfg_attr(feature = "schema", derive(schemars::JsonSchema_repr))]
    #[derive(enum_primitive_derive::Primitive)]
    pub enum ApiErrorCode: u16 = Unknown {
        // Server errors
        50001 = DbError                  = StatusCode::INTERNAL_SERVER_ERROR,
        50002 = JoinError                = StatusCode::INTERNAL_SERVER_ERROR,
        50003 = SemaphoreError           = StatusCode::INTERNAL_SERVER_ERROR,
        50004 = HashError                = StatusCode::INTERNAL_SERVER_ERROR,
        50005 = JsonError                = StatusCode::INTERNAL_SERVER_ERROR,
        50006 = EventEncodingError       = StatusCode::INTERNAL_SERVER_ERROR,
        50007 = InternalError            = StatusCode::INTERNAL_SERVER_ERROR,
        50008 = Utf8ParseError           = StatusCode::INTERNAL_SERVER_ERROR,
        50009 = IOError                  = StatusCode::INTERNAL_SERVER_ERROR,
        50010 = InvalidHeaderValue       = StatusCode::INTERNAL_SERVER_ERROR,
        50011 = XMLError                 = StatusCode::INTERNAL_SERVER_ERROR,
        50012 = RequestError             = StatusCode::INTERNAL_SERVER_ERROR,
        50013 = Unimplemented            = StatusCode::INTERNAL_SERVER_ERROR,
        50014 = BincodeError             = StatusCode::INTERNAL_SERVER_ERROR,
        50015 = CborError                = StatusCode::INTERNAL_SERVER_ERROR,
        50016 = RkyvEncodingError        = StatusCode::INTERNAL_SERVER_ERROR,

        // Client errors
        40001 = AlreadyExists            = StatusCode::CONFLICT,
        40002 = UsernameUnavailable      = StatusCode::BAD_REQUEST,
        40003 = InvalidEmail             = StatusCode::UNAUTHORIZED,
        40004 = InvalidUsername          = StatusCode::UNAUTHORIZED,
        40005 = InvalidPassword          = StatusCode::UNAUTHORIZED,
        40006 = InvalidCredentials       = StatusCode::UNAUTHORIZED,
        40007 = InsufficientAge          = StatusCode::BAD_REQUEST,
        40008 = InvalidDate              = StatusCode::BAD_REQUEST,
        40009 = InvalidContent           = StatusCode::BAD_REQUEST,
        40010 = InvalidName              = StatusCode::BAD_REQUEST,
        40011 = InvalidTopic             = StatusCode::BAD_REQUEST,
        40012 = MissingUploadMetadataHeader  = StatusCode::BAD_REQUEST,
        40013 = MissingAuthorizationHeader   = StatusCode::BAD_REQUEST,
        40014 = NoSession                = StatusCode::UNAUTHORIZED,
        40015 = InvalidAuthFormat        = StatusCode::BAD_REQUEST,
        40016 = HeaderParseError         = StatusCode::UNPROCESSABLE_ENTITY,
        40017 = MissingFilename          = StatusCode::BAD_REQUEST,
        40018 = MissingMime              = StatusCode::BAD_REQUEST,
        40019 = AuthTokenError           = StatusCode::UNPROCESSABLE_ENTITY,
        40020 = Base64DecodeError        = StatusCode::BAD_REQUEST,
        40021 = BodyDeserializeError     = StatusCode::UNPROCESSABLE_ENTITY,
        40022 = QueryParseError          = StatusCode::BAD_REQUEST,
        40023 = UploadError              = StatusCode::BAD_REQUEST,
        40024 = InvalidPreview           = StatusCode::BAD_REQUEST,
        40025 = MimeParseError           = StatusCode::BAD_REQUEST,
        40026 = InvalidImageFormat       = StatusCode::BAD_REQUEST,
        40027 = TOTPRequired             = StatusCode::UNAUTHORIZED,
        40028 = InvalidPreferences       = StatusCode::BAD_REQUEST,
        40029 = TemporarilyDisabled      = StatusCode::FORBIDDEN,
        40030 = InvalidCaptcha           = StatusCode::UNAUTHORIZED,
        40031 = Base85DecodeError        = StatusCode::BAD_REQUEST,
        40032 = WebsocketError           = StatusCode::BAD_REQUEST,
        40033 = MissingContentTypeHeader = StatusCode::BAD_REQUEST,
        40034 = Blocked                  = StatusCode::FORBIDDEN,
        40035 = Banned                   = StatusCode::FORBIDDEN,
        40036 = SearchError              = StatusCode::BAD_REQUEST,

        // Generic HTTP-like error codes
        40400 = BadRequest               = StatusCode::BAD_REQUEST,
        40401 = Unauthorized             = StatusCode::UNAUTHORIZED,
        40404 = NotFound                 = StatusCode::NOT_FOUND,
        40405 = MethodNotAllowed         = StatusCode::METHOD_NOT_ALLOWED,
        40409 = Conflict                 = StatusCode::CONFLICT,
        40413 = RequestEntityTooLarge    = unsafe { StatusCode::from_u16(413).unwrap_unchecked() }, // 413 Request Entity Too Large
        40415 = UnsupportedMediaType     = StatusCode::UNSUPPORTED_MEDIA_TYPE,
        40460 = ChecksumMismatch         = unsafe { StatusCode::from_u16(460).unwrap_unchecked() }, // 460 Checksum Mismatch

        #[serde(other)]
        1 = Unknown = StatusCode::IM_A_TEAPOT,
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind as E;

        Self {
            code: ApiErrorCode::IOError,
            message: Cow::Borrowed(match err.kind() {
                E::AddrInUse => "Address In Use",
                E::AddrNotAvailable => "Address Not Available",
                E::AlreadyExists => "Entity Already Exists",
                E::BrokenPipe => "Broken Pipe",
                E::ConnectionAborted => "Connection Aborted",
                E::ConnectionRefused => "Connection Refused",
                E::ConnectionReset => "Connection Reset",
                E::Interrupted => "Operation Interrupted",
                E::InvalidData => "Invalid Data",
                E::InvalidInput => "Invalid Input Parameter",
                E::NotConnected => "Not Connected",
                E::NotFound => "Entity Not Found",
                E::Other => "Other Error",
                E::OutOfMemory => "Out Of Memory",
                E::PermissionDenied => "Permission Denied",
                E::TimedOut => "Timed Out",
                E::UnexpectedEof => "Unexpected End Of File",
                E::Unsupported => "Unsupported",
                E::WouldBlock => "Operation Would Block",
                E::WriteZero => "Write Zero",
                _ => "Unknown I/O error",
            }),
        }
    }
}
