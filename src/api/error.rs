use std::borrow::Cow;

use enum_primitive_derive::Primitive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: u16,
    pub message: Cow<'static, str>,
}

#[rustfmt::skip]
#[derive(Debug, Primitive, Clone, Copy, PartialEq, Eq, Hash)]
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
    InternalErrorStatic      = 50008,
    Utf8ParseError           = 50009,
    IOError                  = 50010,
    InvalidHeaderValue       = 50011,
    XMLError                 = 50012,
    RequestError             = 50013,
    Unimplemented            = 50014,

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
    AuthTokenParseError      = 40019,
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

    // Generic HTTP-like error codes
    BadRequest               = 40400,
    Unauthorized             = 40401,
    NotFound                 = 40404,
    Conflict                 = 40409,
    RequestEntityTooLarge    = 40413,
    ChecksumMismatch         = 40460,
}