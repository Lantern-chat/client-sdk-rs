use super::*;

use crate::util::fixed::FixedStr;

pub type BearerToken = FixedStr<28>;
pub type BotToken = FixedStr<64>;

const MAX_LENGTH: usize = 68; // ("Bearer ".len() + 28).max("Bot ".len() + 64)

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum AuthToken {
    Bearer(BearerToken),
    Bot(BotToken),
}

impl AuthToken {
    pub fn raw_header(&self) -> arrayvec::ArrayString<{ MAX_LENGTH }> {
        let (prefix, value) = match self {
            AuthToken::Bearer(ref token) => ("Bearer ", token.as_ref()),
            AuthToken::Bot(ref token) => ("Bot ", token.as_ref()),
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
