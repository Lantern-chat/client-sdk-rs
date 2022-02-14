use std::sync::Arc;

use headers::{ContentType, HeaderMapExt, HeaderValue};
use http::Method;
use reqwest::{Request, Url};

mod error;
pub use error::DriverError;

use crate::{
    api::{Command, CommandFlags},
    models::{AuthToken, Snowflake},
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
    pub(crate) auth: Option<Arc<HeaderValue>>,
}

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
        Ok(Self::new_from_raw(uri, generic_client().build()?))
    }

    pub fn new_from_raw(uri: Arc<String>, client: reqwest::Client) -> Self {
        Driver {
            inner: client,
            uri,
            encoding: Encoding::Json,
            auth: None,
        }
    }

    pub fn set_token(&mut self, token: Option<AuthToken>) -> Result<(), DriverError> {
        self.auth = match token {
            Some(token) => Some(Arc::new(token.headervalue()?)),
            None => None,
        };

        Ok(())
    }

    fn add_auth_header(&self, req: &mut Request) -> Result<(), DriverError> {
        match self.auth {
            Some(ref auth) => {
                req.headers_mut().insert("Authorization", (**auth).clone());
            }
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

#[allow(unused_variables)]
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

    #[allow(unused_mut)]
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
            .header("Authorization", auth)
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
