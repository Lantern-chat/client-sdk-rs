use std::borrow::Cow;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub message: Cow<'static, str>,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema_repr))]
#[derive(enum_primitive_derive::Primitive)]
#[repr(u16)]
pub enum ApiErrorCode {
    // Server errors
    DbError                  = 50001,
    JoinError                = 50002,
    SemaphoreError           = 50003,
    HashError                = 50004,
    JsonError                = 50005,
    EventEncodingError       = 50006,
    InternalError            = 50007,
    Utf8ParseError           = 50008,
    IOError                  = 50009,
    InvalidHeaderValue       = 50010,
    XMLError                 = 50011,
    RequestError             = 50012,
    Unimplemented            = 50013,
    BincodeError             = 50014,

    // Client errors
    AlreadyExists            = 40001,
    UsernameUnavailable      = 40002,
    InvalidEmail             = 40003,
    InvalidUsername          = 40004,
    InvalidPassword          = 40005,
    InvalidCredentials       = 40006,
    InsufficientAge          = 40007,
    InvalidDate              = 40008,
    InvalidContent           = 40009,
    InvalidName              = 40010,
    InvalidTopic             = 40011,
    MissingUploadMetadataHeader  = 40012,
    MissingAuthorizationHeader   = 40013,
    NoSession                = 40014,
    InvalidAuthFormat        = 40015,
    HeaderParseError         = 40016,
    MissingFilename          = 40017,
    MissingMime              = 40018,
    AuthTokenError           = 40019,
    Base64DecodeError        = 40020,
    BodyDeserializeError     = 40021,
    QueryParseError          = 40022,
    UploadError              = 40023,
    InvalidPreview           = 40024,
    MimeParseError           = 40025,
    InvalidImageFormat       = 40026,
    TOTPRequired             = 40027,
    InvalidPreferences       = 40028,
    TemporarilyDisabled      = 40029,
    InvalidCaptcha           = 40030,
    Base85DecodeError        = 40031,
    WebsocketError           = 40032,
    MissingContentTypeHeader = 40033,
    Blocked                  = 40034,

    // Generic HTTP-like error codes
    BadRequest               = 40400,
    Unauthorized             = 40401,
    NotFound                 = 40404,
    MethodNotAllowed         = 40405,
    Conflict                 = 40409,
    RequestEntityTooLarge    = 40413,
    UnsupportedMediaType     = 40415,
    ChecksumMismatch         = 40460,

    #[serde(other)]
    Unknown = 1,
}
