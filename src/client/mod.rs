use headers::{ContentType, HeaderMapExt};
use http::Method;
use reqwest::{Request, Url};

use crate::{
    api::{error::ApiError, Command, CommandFlags},
    models::SmolToken,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    Json,

    #[cfg(feature = "msgpack")]
    MsgPack,
}

#[derive(Clone)]
pub struct Client {
    pub inner: reqwest::Client,
    pub encoding: Encoding,
    pub domain: &'static str,
    pub secure: bool,
    pub token: Option<SmolToken>,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
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

    #[error("Invalid Bearer Token")]
    InvalidBearerToken,

    #[error("Client Error: {0:?}")]
    ApiError(ApiError),

    #[error("Generic Client Error: {0}")]
    GenericClientError(http::StatusCode),
}

impl Client {
    pub async fn execute<CMD: Command>(&self, cmd: CMD) -> Result<CMD::Result, ClientError> {
        let mut path = format!(
            "http{}://{}/api/v1/",
            if self.secure { "s" } else { "" },
            self.domain
        );

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
            match self.token {
                Some(ref token) => {
                    req.headers_mut()
                        .typed_insert(match headers::Authorization::bearer(token) {
                            Ok(header) => header,
                            Err(_) => return Err(ClientError::InvalidBearerToken),
                        });
                }
                None => panic!("Cannot execute authorized command without auth token!"),
            }
        }

        let response = self.inner.execute(req).await?;

        let status = response.status();
        let ct = response.headers().typed_get::<ContentType>();
        let body = response.bytes().await?;

        if !status.is_success() {
            return Err(match deserialize_ct(&body, ct) {
                Ok(api_error) => ClientError::ApiError(api_error),
                Err(_) => ClientError::GenericClientError(status),
            });
        }

        if body.len() == 0 || std::mem::size_of::<CMD::Result>() == 0 {
            // if Result is a zero-size type, this is likely optimized away entirely.
            // Otherwise, if the body is empty, try to deserialize an empty object
            return Ok(serde_json::from_slice(b"{}").unwrap());
        }

        deserialize_ct(&body, ct)
    }
}

fn deserialize_ct<T>(body: &[u8], ct: Option<ContentType>) -> Result<T, ClientError>
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
