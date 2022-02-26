use super::*;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{Read, Write},
    mem::size_of,
    num::NonZeroU64,
};

type HmacDigest = [u8; 20];

/// Decomposed bot token with its component parts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SplitBotToken {
    pub id: Snowflake,
    pub ts: u64,
    pub hmac: HmacDigest,
}

impl SplitBotToken {
    pub const SPLIT_BOT_TOKEN_SIZE: usize =
        size_of::<Snowflake>() + size_of::<u64>() + size_of::<HmacDigest>();

    #[inline]
    pub fn to_bytes(&self) -> [u8; Self::SPLIT_BOT_TOKEN_SIZE] {
        let mut bytes = [0u8; Self::SPLIT_BOT_TOKEN_SIZE];

        let mut w: &mut [u8] = &mut bytes;

        unsafe {
            w.write_u64::<LittleEndian>(self.id.to_u64()).unwrap_unchecked();
            w.write_u64::<LittleEndian>(self.ts).unwrap_unchecked();
            w.write(&self.hmac).unwrap_unchecked();
        }

        bytes
    }

    pub fn format(&self) -> BotToken {
        let mut token;
        unsafe {
            token = BotToken::zeroized();
            let res =
                base64::encode_config_slice(self.to_bytes(), base64::STANDARD_NO_PAD, token.as_bytes_mut());
            debug_assert_eq!(res, BotToken::LEN);
        }

        token
    }
}

impl TryFrom<&[u8]> for SplitBotToken {
    type Error = InvalidAuthToken;

    #[inline]
    fn try_from(mut bytes: &[u8]) -> Result<SplitBotToken, InvalidAuthToken> {
        if bytes.len() != Self::SPLIT_BOT_TOKEN_SIZE {
            return Err(InvalidAuthToken);
        }

        let raw_id;
        let ts;
        let mut hmac: HmacDigest = [0; 20];

        unsafe {
            raw_id = bytes.read_u64::<LittleEndian>().unwrap_unchecked();
            ts = bytes.read_u64::<LittleEndian>().unwrap_unchecked();
            bytes.read_exact(&mut hmac).unwrap_unchecked();
        }

        let id = match NonZeroU64::new(raw_id) {
            Some(id) => Snowflake(id),
            None => return Err(InvalidAuthToken),
        };

        Ok(SplitBotToken { id, ts, hmac })
    }
}

impl FromStr for SplitBotToken {
    type Err = InvalidAuthToken;

    fn from_str(s: &str) -> Result<SplitBotToken, InvalidAuthToken> {
        if s.len() != BotToken::LEN {
            return Err(InvalidAuthToken);
        }

        let mut bytes = [0; Self::SPLIT_BOT_TOKEN_SIZE];
        if base64::decode_config_slice(s, base64::STANDARD_NO_PAD, &mut bytes).is_err() {
            return Err(InvalidAuthToken);
        }

        SplitBotToken::try_from(&bytes[..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splitbottoken_bytes() {
        let token = SplitBotToken {
            id: Snowflake::null(),
            ts: 0,
            hmac: [u8::MAX; 20],
        };

        let bytes = token.to_bytes();

        assert_eq!(token, SplitBotToken::try_from(&bytes[..]).unwrap());

        println!("{}", token.format());
    }
}
