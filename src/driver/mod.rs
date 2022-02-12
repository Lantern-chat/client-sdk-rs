use std::sync::Arc;

use headers::{
    authorization::{Authorization, Bearer, Credentials},
    ContentType, HeaderMapExt,
};
use http::Method;
use reqwest::{Request, Url};

use crate::{
    api::{error::ApiError, Command, CommandFlags},
    models::{SmolToken, Snowflake},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    Json,

    #[cfg(feature = "msgpack")]
    MsgPack,
}

#[derive(Clone)]
pub struct Driver {
    pub(crate) inner: reqwest::Client,
    pub(crate) encoding: Encoding,
    pub(crate) uri: Arc<String>,
    pub(crate) auth: Option<Arc<Authorization<Bearer>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("Reqwest Error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Format Error")]
    FormatError(#[from] std::fmt::Error),

    #[error("Url Parse Error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Url Encoding Error: {0}")]
    UrlEncodingError(#[from] serde_urlencoded::ser::Error),

    #[error("JSON Error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[cfg(feature = "msgpack")]
    #[error("MsgPack Encode Error: {0}")]
    MsgPackEncodeError(#[from] rmp_serde::encode::Error),

    #[cfg(feature = "msgpack")]
    #[error("MsgPack Decode Error: {0}")]
    MsgPackDecodeError(#[from] rmp_serde::decode::Error),

    #[error("Api Error: {0:?}")]
    ApiError(ApiError),

    #[error("Generic Driver Error: {0}")]
    GenericDriverError(http::StatusCode),

    #[error("Missing Authorization")]
    MissingAuthorization,

    #[error("Parse Int Error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Header Parse Error: {0}")]
    HeaderParseError(#[from] http::header::ToStrError),
}

#[derive(Debug, thiserror::Error)]
#[error("InvalidBearerToken")]
pub struct InvalidBearerToken;

pub(crate) fn generic_client() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Lantern Driver SDK)")
        .gzip(true)
        .deflate(true)
        .brotli(true)
        .http2_adaptive_window(true)
}

impl Driver {
    pub fn new_static(uri: &'static str) -> Result<Self, DriverError> {
        Self::new(Arc::new(uri.to_owned()))
    }

    pub fn new(uri: Arc<String>) -> Result<Self, DriverError> {
        Ok(Self::new_from_raw(uri, generic_client().build()?, None))
    }

    pub fn new_from_raw(
        uri: Arc<String>,
        client: reqwest::Client,
        auth: Option<Arc<Authorization<Bearer>>>,
    ) -> Self {
        Driver {
            inner: client,
            uri,
            encoding: Encoding::Json,
            auth,
        }
    }

    pub fn set_token(&mut self, token: Option<SmolToken>) -> Result<(), InvalidBearerToken> {
        self.auth = match token {
            Some(token) => match Authorization::bearer(&token) {
                Ok(auth) => Some(Arc::new(auth)),
                Err(_) => return Err(InvalidBearerToken),
            },
            None => None,
        };

        Ok(())
    }

    fn add_auth_header(&self, req: &mut Request) -> Result<(), DriverError> {
        match self.auth {
            Some(ref auth) => req.headers_mut().typed_insert((**auth).clone()),
            None => return Err(DriverError::MissingAuthorization),
        }

        Ok(())
    }

    pub async fn execute<CMD: Command>(&self, cmd: CMD) -> Result<CMD::Result, DriverError> {
        let mut path = format!("{}/api/v1/", self.uri);

        // likely inlined, simple
        cmd.format_path(&mut path)?;

        let mut req = Request::new(CMD::METHOD, Url::parse(&path)?);

        // likely inlined, often no-ops
        cmd.add_headers(req.headers_mut());

        let body_size_hint = cmd.body_size_hint();

        // if there is a body to serialize
        if CMD::FLAGS.contains(CommandFlags::HAS_BODY) && body_size_hint > 0 {
            match CMD::METHOD {
                // for methods without bodies, the "body" is treated as query parameters
                Method::GET | Method::OPTIONS | Method::HEAD | Method::CONNECT | Method::TRACE => {
                    let url = req.url_mut();

                    {
                        let mut pairs = url.query_pairs_mut();
                        cmd.serialize_body(serde_urlencoded::Serializer::new(&mut pairs))?;
                    }

                    if let Some("") = url.query() {
                        url.set_query(None);
                    }
                }
                _ => {
                    let mut body = Vec::with_capacity(body_size_hint.max(128));

                    match self.encoding {
                        Encoding::Json => {
                            cmd.serialize_body(&mut serde_json::Serializer::new(&mut body))?;

                            req.headers_mut().typed_insert(ContentType::json());
                        }

                        #[cfg(feature = "msgpack")]
                        Encoding::MsgPack => {
                            cmd.serialize_body(&mut rmp_serde::Serializer::new(&mut body))?;

                            req.headers_mut()
                                .typed_insert(ContentType::from(mime::APPLICATION_MSGPACK));
                        }
                    }

                    *req.body_mut() = Some(body.into());
                }
            }
        }

        if CMD::FLAGS.contains(CommandFlags::AUTHORIZED) {
            self.add_auth_header(&mut req)?;
        }

        let response = self.inner.execute(req).await?;

        let status = response.status();
        let ct = response.headers().typed_get::<ContentType>();
        let body = response.bytes().await?;

        if !status.is_success() {
            return Err(match deserialize_ct(&body, ct) {
                Ok(api_error) => DriverError::ApiError(api_error),
                Err(_) => DriverError::GenericDriverError(status),
            });
        }

        if body.len() == 0 || std::mem::size_of::<CMD::Result>() == 0 {
            // if Result is a zero-size type, this is likely optimized away entirely.
            // Otherwise, if the body is empty, try to deserialize an empty object
            return Ok(serde_json::from_slice(b"{}")?);
        }

        deserialize_ct(&body, ct)
    }
}

fn deserialize_ct<T>(body: &[u8], ct: Option<ContentType>) -> Result<T, DriverError>
where
    T: serde::de::DeserializeOwned,
{
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum BodyType {
        Json,

        #[cfg(feature = "msgpack")]
        MsgPack,
    }

    let mut kind = BodyType::Json;

    #[cfg(feature = "msgpack")]
    if ct == Some(ContentType::from(mime::APPLICATION_MSGPACK)) {
        kind = BodyType::MsgPack;
    }

    Ok(match kind {
        BodyType::Json => serde_json::from_slice(body)?,

        #[cfg(feature = "msgpack")]
        BodyType::MsgPack => rmp_serde::from_slice(body)?,
    })
}

impl Driver {
    pub async fn patch_file(
        &self,
        file_id: Snowflake,
        checksum: u32,
        offset: u64,
        body: reqwest::Body,
    ) -> Result<u64, DriverError> {
        let path = format!("{}/api/v1/file/{}", self.uri, file_id);

        let auth = match self.auth {
            Some(ref auth) => (**auth).clone(),
            None => return Err(DriverError::MissingAuthorization),
        };

        let response = self
            .inner
            .patch(path)
            .header("Authorization", auth.0.encode())
            .header("Upload-Offset", offset)
            .header(
                "Upload-Checksum",
                format!("crc32 {}", base64::encode(&checksum.to_be_bytes())),
            )
            .header("Content-Type", "application/offset+octet-stream")
            .body(body)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            if let Some(offset) = response.headers().get("Upload-Offset") {
                return Ok(offset.to_str()?.parse()?);
            }
        }

        let ct = response.headers().typed_get::<ContentType>();
        let body = response.bytes().await?;

        return Err(match deserialize_ct(&body, ct) {
            Ok(api_error) => DriverError::ApiError(api_error),
            Err(_) => DriverError::GenericDriverError(status),
        });
    }
}
