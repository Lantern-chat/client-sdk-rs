use std::sync::Arc;

use reqwest::{
    header::{HeaderName, HeaderValue},
    Method, Request, Url,
};

mod error;
pub use error::DriverError;

use crate::{
    api::{Command, CommandFlags},
    models::{AuthToken, FileId},
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    #[default]
    JSON,

    #[cfg(feature = "cbor")]
    CBOR,
}

#[derive(Clone)]
pub struct Driver {
    pub(crate) inner: reqwest::Client,
    pub(crate) encoding: Encoding,
    pub(crate) uri: Arc<str>,
    pub(crate) auth: Option<Arc<(AuthToken, HeaderValue)>>,
}

pub(crate) fn generic_client() -> reqwest::ClientBuilder {
    #[allow(unused_mut)]
    let mut builder = reqwest::Client::builder()
        .user_agent("Lantern SDK/1.0 (bot; https://github.com/Lantern-chat)")
        .gzip(true)
        .deflate(true)
        .http2_adaptive_window(true);

    #[cfg(feature = "brotli")]
    {
        builder = builder.brotli(true);
    }

    builder
}

impl Driver {
    pub fn new(uri: &str) -> Result<Self, DriverError> {
        Self::new_shared(Arc::from(uri))
    }

    pub fn new_shared(uri: Arc<str>) -> Result<Self, DriverError> {
        Ok(Self::new_from_raw(uri, generic_client().build()?))
    }

    pub fn new_from_raw(uri: Arc<str>, client: reqwest::Client) -> Self {
        Driver {
            inner: client,
            uri,
            encoding: Encoding::JSON,
            auth: None,
        }
    }

    pub fn set_token(&mut self, token: Option<AuthToken>) -> Result<(), DriverError> {
        self.auth = match token {
            Some(token) => Some(Arc::new((token, token.headervalue()?))),
            None => None,
        };

        Ok(())
    }

    fn add_auth_header(&self, req: &mut Request) -> Result<(), DriverError> {
        match self.auth {
            Some(ref auth) => {
                req.headers_mut().insert(HeaderName::from_static("authorization"), auth.1.clone());
            }
            None => return Err(DriverError::MissingAuthorization),
        }

        Ok(())
    }

    /// Same as [`execute`](Driver::execute), but will return an `Option` if the API returned 404 Not Found.
    pub async fn execute_opt<CMD: Command>(&self, cmd: CMD) -> Result<Option<CMD::Result>, DriverError> {
        match self.execute(cmd).await {
            Ok(value) => Ok(Some(value)),
            Err(e) if e.is_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Execute the given command, taking care of all body and query parameters automatically.
    ///
    /// If you would like an `Option` for not-found values, use [`execute_opt`](Driver::execute_opt) instead.
    pub async fn execute<CMD: Command>(&self, cmd: CMD) -> Result<CMD::Result, DriverError> {
        let mut path = format!("{}/api/v1/", self.uri);

        // likely inlined, simple
        cmd.format_path(&mut path)?;

        let mut req = Request::new(CMD::HTTP_METHOD, Url::parse(&path)?);

        // likely inlined, often no-ops
        cmd.add_headers(req.headers_mut());

        let body_size_hint = cmd.body_size_hint();

        // if there is a body to serialize
        if CMD::FLAGS.contains(CommandFlags::HAS_BODY) && body_size_hint > 0 {
            match CMD::HTTP_METHOD {
                // for methods without bodies, the "body" is treated as query parameters
                Method::GET | Method::OPTIONS | Method::HEAD | Method::CONNECT | Method::TRACE => {
                    let url = req.url_mut();

                    {
                        use serde::Serialize;

                        let mut pairs = url.query_pairs_mut();
                        cmd.body().serialize(serde_urlencoded::Serializer::new(&mut pairs))?;
                    }

                    if let Some("") = url.query() {
                        url.set_query(None);
                    }
                }
                _ => {
                    let mut body = Vec::with_capacity(body_size_hint.max(128));

                    // TODO: Fix when reqwest updates
                    match self.encoding {
                        Encoding::JSON => {
                            serde_json::to_writer(&mut body, cmd.body())?;

                            req.headers_mut().insert(
                                HeaderName::from_static("content-type"),
                                HeaderValue::from_static("application/json"),
                            );
                        }

                        #[cfg(feature = "cbor")]
                        Encoding::CBOR => {
                            ciborium::ser::into_writer(cmd.body(), &mut body)?;

                            req.headers_mut().insert(
                                HeaderName::from_static("content-type"),
                                HeaderValue::from_static("application/cbor"),
                            );
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
        let ct = response.headers().get(HeaderName::from_static("content-type")).cloned();
        let body = response.bytes().await?;

        if !status.is_success() {
            return Err(match deserialize_ct(&body, ct) {
                Ok(api_error) => DriverError::ApiError(api_error),
                Err(_) => DriverError::GenericDriverError(status),
            });
        }

        if body.is_empty() || core::mem::size_of::<CMD::Result>() == 0 {
            // if Result is a zero-size type, this is likely optimized away entirely.
            // Otherwise, if the body is empty, try to deserialize an empty object
            return Ok(serde_json::from_slice(b"{}")?);
        }

        deserialize_ct(&body, ct)
    }
}

// lazy_static::lazy_static! {
//     pub(crate) static ref APPLICATION_CBOR: ContentType = ContentType::from("application/cbor".parse::<mime::Mime>().unwrap());
// }

#[allow(unused_variables)]
fn deserialize_ct<T>(body: &[u8], ct: Option<HeaderValue>) -> Result<T, DriverError>
where
    T: serde::de::DeserializeOwned,
{
    #[allow(unused_mut)]
    let mut kind = Encoding::JSON;

    #[cfg(feature = "cbor")]
    if let Some(ct) = ct {
        if ct.as_bytes() == b"application/cbor" {
            kind = Encoding::CBOR;
        }
    }

    Ok(match kind {
        Encoding::JSON => serde_json::from_slice(body)?,

        #[cfg(feature = "cbor")]
        Encoding::CBOR => ciborium::de::from_reader(body)?,
    })
}

use base64::engine::{general_purpose::STANDARD, Engine};

impl Driver {
    pub async fn patch_file(&self, file_id: FileId, offset: u64, chunk: bytes::Bytes) -> Result<u64, DriverError> {
        let auth = match self.auth {
            Some(ref auth) => auth.1.clone(),
            None => return Err(DriverError::MissingAuthorization),
        };

        let path = format!("{}/api/v1/file/{}", self.uri, file_id);

        let checksum = crc32fast::hash(&chunk);

        let response = self
            .inner
            .patch(path)
            .header(HeaderName::from_static("authorization"), auth)
            .header(HeaderName::from_static("upload-offset"), offset)
            .header(
                HeaderName::from_static("upload-checksum"),
                format!("crc32 {}", STANDARD.encode(checksum.to_be_bytes())),
            )
            .header(
                HeaderName::from_static("content-type"),
                HeaderValue::from_static("application/offset+octet-stream"),
            )
            .body(chunk)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            if let Some(offset) = response.headers().get(HeaderName::from_static("upload-offset")) {
                return Ok(offset.to_str().expect("Fix this").parse()?);
            }
        }

        let ct = response.headers().get(HeaderName::from_static("content-type")).cloned();
        let body = response.bytes().await?;

        Err(match deserialize_ct(&body, ct) {
            Ok(api_error) => DriverError::ApiError(api_error),
            Err(_) => DriverError::GenericDriverError(status),
        })
    }
}
