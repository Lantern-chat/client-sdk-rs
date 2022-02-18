use std::fmt;
use std::str::FromStr;

use super::*;

use crate::util::fixed::FixedStr;

pub type BearerToken = FixedStr<28>;
pub type BotToken = FixedStr<64>;

const BEARER_PREFIX: &str = "Bearer ";
const BOT_PREFIX: &str = "Bot ";

const BEARER_HEADER_LENGTH: usize = BEARER_PREFIX.len() + BearerToken::LEN;
const BOT_HEADER_LENGTH: usize = BOT_PREFIX.len() + BotToken::LEN;

const MAX_LENGTH: usize = BOT_HEADER_LENGTH; // We know this one is larger...

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum AuthToken {
    Bearer(BearerToken),
    Bot(BotToken),
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid Auth Token")]
pub struct InvalidAuthToken;

impl AuthToken {
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
        http::HeaderValue::from_str(&self.raw_header())
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

        if value.len() == BEARER_HEADER_LENGTH && value.starts_with(BEARER_PREFIX) {
            return Ok(AuthToken::Bearer(BearerToken::new(&value[BEARER_PREFIX.len()..])));
        }

        if value.len() == BOT_HEADER_LENGTH && value.starts_with(BOT_PREFIX) {
            return Ok(AuthToken::Bot(BotToken::new(&value[BOT_PREFIX.len()..])));
        }

        Err(InvalidAuthToken)
    }
}

impl std::ops::Deref for AuthToken {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        match self {
            AuthToken::Bearer(ref token) => token.as_ref(),
            AuthToken::Bot(ref token) => token.as_ref(),
        }
    }
}

mod serde {
    use super::{AuthToken, BearerToken, BotToken};

    use std::fmt;

    use serde::de::{self, Deserialize, Deserializer, Visitor};

    impl<'de> Deserialize<'de> for AuthToken {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct AuthTokenVisitor;

            impl<'de> Visitor<'de> for AuthTokenVisitor {
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
