use core::{fmt, str::FromStr};

use super::*;

// NOTE: Keep these in sync with the lengths in client-sdk-ts

/// Fixed-length string with a maximum length of 28 bytes for bearer tokens.
pub type BearerToken = FixedStr<28>;
/// Fixed-length string with a maximum length of 48 bytes for bot tokens.
pub type BotToken = FixedStr<48>;

const BEARER_PREFIX: &str = "Bearer ";
const BOT_PREFIX: &str = "Bot ";

const BEARER_HEADER_LENGTH: usize = BEARER_PREFIX.len() + BearerToken::LEN;
const BOT_HEADER_LENGTH: usize = BOT_PREFIX.len() + BotToken::LEN;

const MAX_LENGTH: usize = {
    if BEARER_HEADER_LENGTH < BOT_HEADER_LENGTH {
        BOT_HEADER_LENGTH
    } else {
        BEARER_HEADER_LENGTH
    }
};

/// Raw base64-encoded auth tokens for users and bots.
#[derive(Debug, Clone, Copy, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "ts", derive(ts_bindgen::TypeScriptDef))]
#[serde(untagged)]
pub enum AuthToken {
    /// Bearer token for users
    Bearer(BearerToken),
    /// Bot token for bots
    Bot(BotToken),
}

// TODO: rkyv AuthToken without discriminator by using token length

#[derive(Debug)]
pub struct InvalidAuthToken;

impl fmt::Display for InvalidAuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Invalid Auth Token")
    }
}

impl core::error::Error for InvalidAuthToken {}

impl AuthToken {
    #[must_use]
    pub fn raw_header(&self) -> arrayvec::ArrayString<{ MAX_LENGTH }> {
        let (prefix, value) = match self {
            AuthToken::Bearer(ref token) => (BEARER_PREFIX, token.as_ref()),
            AuthToken::Bot(ref token) => (BOT_PREFIX, token.as_ref()),
        };

        let mut buffer = arrayvec::ArrayString::new();

        buffer.push_str(prefix);
        buffer.push_str(value);

        buffer
    }

    #[cfg(feature = "http")]
    pub fn headervalue(&self) -> Result<http::HeaderValue, http::header::InvalidHeaderValue> {
        http::HeaderValue::from_str(&self.raw_header()).map(|mut h| {
            h.set_sensitive(true);
            h
        })
    }

    pub fn from_header(mut value: &str) -> Result<Self, InvalidAuthToken> {
        value = value.trim();

        if value.len() == BEARER_HEADER_LENGTH && value.starts_with(BEARER_PREFIX) {
            return Ok(AuthToken::Bearer(BearerToken::new(&value[BEARER_PREFIX.len()..])));
        }

        if value.len() == BOT_HEADER_LENGTH && value.starts_with(BOT_PREFIX) {
            return Ok(AuthToken::Bot(BotToken::new(&value[BOT_PREFIX.len()..])));
        }

        Err(InvalidAuthToken)
    }
}

impl fmt::Display for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.raw_header())
    }
}

impl FromStr for AuthToken {
    type Err = InvalidAuthToken;

    fn from_str(mut value: &str) -> Result<Self, InvalidAuthToken> {
        value = value.trim();

        if value.len() == BearerToken::LEN {
            return Ok(AuthToken::Bearer(BearerToken::new(value)));
        }

        if value.len() == BotToken::LEN {
            return Ok(AuthToken::Bot(BotToken::new(value)));
        }

        Err(InvalidAuthToken)
    }
}

impl core::ops::Deref for AuthToken {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        match self {
            AuthToken::Bearer(ref token) => token.as_ref(),
            AuthToken::Bot(ref token) => token.as_ref(),
        }
    }
}

mod serde_impl {
    use super::{AuthToken, BearerToken, BotToken};

    use core::fmt;

    use serde::de::{self, Deserialize, Deserializer, Visitor};

    impl<'de> Deserialize<'de> for AuthToken {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct AuthTokenVisitor;

            impl Visitor<'_> for AuthTokenVisitor {
                type Value = AuthToken;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "a string of length {} or {}", BearerToken::LEN, BotToken::LEN)
                }

                // fast path that doesn't have to fail one of them first
                fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                    if value.len() == BearerToken::LEN {
                        Ok(AuthToken::Bearer(BearerToken::new(value)))
                    } else if value.len() == BotToken::LEN {
                        Ok(AuthToken::Bot(BotToken::new(value)))
                    } else {
                        Err(E::invalid_length(value.len(), &self))
                    }
                }
            }

            deserializer.deserialize_str(AuthTokenVisitor)
        }
    }
}
