use std::fmt;

/// Fixed-size String that can *only* be a given length, no more or less, exactly N bytes
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FixedStr<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> AsRef<str> for FixedStr<N> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        // SAFETY: Can only be created from checked utf-8 in the first place
        unsafe { std::str::from_utf8_unchecked(&self.data) }
    }
}

impl<const N: usize> AsRef<[u8]> for FixedStr<N> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<const N: usize> AsMut<str> for FixedStr<N> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut str {
        // SAFETY: Can only be created from checked utf-8 in the first place
        unsafe { std::str::from_utf8_unchecked_mut(&mut self.data) }
    }
}

impl<const N: usize> std::ops::Deref for FixedStr<N> {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        self.as_ref()
    }
}

impl<const N: usize> std::ops::DerefMut for FixedStr<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut()
    }
}

impl<const N: usize> FixedStr<N> {
    pub const LEN: usize = N;

    /// Construct a new [FixedStr] from a given ASCII character repeated for the entire length
    pub const fn repeat_ascii(c: char) -> FixedStr<N> {
        if !c.is_ascii() {
            panic!("Non-ASCII character given");
        }

        FixedStr { data: [c as u8; N] }
    }

    /// Create a string of \0 values.
    pub const unsafe fn zeroized() -> FixedStr<N> {
        FixedStr { data: [0; N] }
    }

    /// Construct a new [FixedStr] from a `&str`, panics if the length is not exactly correct.
    #[inline]
    pub const fn new(s: &str) -> FixedStr<N> {
        if s.len() != N {
            panic!("FixedStr length must be the exact length");
        }

        let mut data = [0; N];

        let mut i = 0;
        while i < N {
            data[i] = s.as_bytes()[i];
            i += 1;
        }

        FixedStr { data }
    }

    #[inline]
    pub const fn try_from(s: &str) -> Result<FixedStr<N>, ()> {
        if s.len() != N {
            return Err(());
        }

        Ok(Self::new(s))
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl<const N: usize> fmt::Debug for FixedStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FixedStr").field(&self.as_str()).finish()
    }
}

impl<const N: usize> fmt::Display for FixedStr<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

mod serde_impl {
    use super::FixedStr;

    use std::fmt;
    use std::marker::PhantomData;

    use serde::de::{self, Deserialize, Deserializer, Visitor};
    use serde::ser::{Serialize, Serializer};

    impl<const N: usize> Serialize for FixedStr<N> {
        #[inline]
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.as_str().serialize(serializer)
        }
    }

    impl<'de, const N: usize> Deserialize<'de> for FixedStr<N> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct FixedStrVisitor<const N: usize>(PhantomData<[(); N]>);

            impl<'de, const N: usize> Visitor<'de> for FixedStrVisitor<N> {
                type Value = FixedStr<N>;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "a string of exactly {} bytes", N)
                }

                #[inline]
                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    FixedStr::try_from(value).map_err(|_| E::invalid_length(value.len(), &self))
                }
            }

            deserializer.deserialize_str(FixedStrVisitor(PhantomData))
        }
    }
}
