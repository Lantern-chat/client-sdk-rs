// This file is dual-licensed under either the MIT or Apache 2.0 license.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// You may choose either license to govern your use of this file.
// Any types re-exported from this file also fall under the same license.

//! Small fixed-size string type that can only be a given length, no more or less, exactly `N` bytes.

use core::fmt;

/// Fixed-size String that can *only* be a given length, no more or less, exactly N bytes
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[must_use = "Not using the FixedStr is wasteful"]
#[repr(transparent)]
pub struct FixedStr<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> From<FixedStr<N>> for smol_str::SmolStr {
    #[inline(always)]
    fn from(fixed: FixedStr<N>) -> Self {
        Self::new(fixed.as_str())
    }
}

impl<const N: usize> AsRef<str> for FixedStr<N> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        // SAFETY: Can only be created from checked utf-8 in the first place
        unsafe { core::str::from_utf8_unchecked(&self.data) }
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
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.data) }
    }
}

impl<const N: usize> core::ops::Deref for FixedStr<N> {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        self.as_ref()
    }
}

impl<const N: usize> core::ops::DerefMut for FixedStr<N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut()
    }
}

impl<const N: usize> FixedStr<N> {
    pub const LEN: usize = N;

    /// Construct a new [FixedStr] from a given ASCII character repeated for the entire length
    ///
    /// # Panics
    /// * if the character is not ASCII
    pub const fn repeat_ascii(c: char) -> FixedStr<N> {
        if !c.is_ascii() {
            panic!("Non-ASCII character given");
        }

        FixedStr { data: [c as u8; N] }
    }

    /// Construct a new [FixedStr] from a [`&str`](str) if the length is correct.
    ///
    /// # Panics
    /// * if the length is not exactly correct.
    #[inline]
    pub const fn new(s: &str) -> FixedStr<N> {
        if s.len() != N {
            panic!("FixedStr length must be the exact length");
        }

        let mut data = [0; N];
        let src = s.as_bytes();

        // must use while-loop in const function
        let mut i = 0;
        while i < N {
            data[i] = src[i];
            i += 1;
        }

        FixedStr { data }
    }

    /// Construct a new [FixedStr] from a byte array, assuming valid utf-8.
    ///
    /// # Safety
    ///
    /// This function is unsafe because in release-mode it does not check if the
    /// bytes are valid utf-8.
    ///
    /// # Panics
    ///
    /// * In debug mode, this function will panic if the bytes are not valid utf-8.
    /// * In release mode, this function will not check the bytes.
    #[inline]
    pub const unsafe fn from_bytes(bytes: [u8; N]) -> FixedStr<N> {
        #[cfg(debug_assertions)]
        if core::str::from_utf8(&bytes).is_err() {
            panic!("Invalid utf-8 bytes given");
        }

        FixedStr { data: bytes }
    }

    /// Construct a new [FixedStr] from a [`&str`](str) if the length is correct.
    #[inline]
    #[must_use]
    pub const fn try_from(s: &str) -> Option<FixedStr<N>> {
        if s.len() != N {
            return None;
        }

        Some(Self::new(s))
    }

    #[inline(always)]
    #[must_use]
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

#[cfg(feature = "ts")]
const _: () = {
    use ts_bindgen::{TypeRegistry, TypeScriptDef, TypeScriptType};

    impl<const N: usize> TypeScriptDef for FixedStr<N> {
        fn register(_: &mut TypeRegistry) -> TypeScriptType {
            TypeScriptType::String(None)
        }
    }
};

//#[cfg(feature = "serde")]
const _: () = {
    use core::fmt;
    use core::marker::PhantomData;

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

            impl<const N: usize> Visitor<'_> for FixedStrVisitor<N> {
                type Value = FixedStr<N>;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "a string of exactly {N} bytes")
                }

                #[inline]
                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    FixedStr::try_from(value).ok_or_else(|| E::invalid_length(value.len(), &self))
                }
            }

            deserializer.deserialize_str(FixedStrVisitor(PhantomData))
        }
    }
};

#[cfg(feature = "rkyv")]
const _: () = {
    use core::{slice::from_raw_parts, str::from_utf8};

    use rkyv::{
        bytecheck::CheckBytes,
        place::Place,
        rancor::{Fallible, ResultExt, Source},
        traits::{CopyOptimization, NoUndef},
        Archive, Deserialize, Portable, Serialize,
    };

    unsafe impl<const N: usize> Portable for FixedStr<N> {}
    unsafe impl<const N: usize> NoUndef for FixedStr<N> {}

    impl<const N: usize> Archive for FixedStr<N> {
        type Archived = FixedStr<N>;
        type Resolver = ();

        // SAFETY: We know that FixedStr is a valid type to copy as-is
        const COPY_OPTIMIZATION: CopyOptimization<Self> = unsafe { CopyOptimization::enable() };

        #[inline]
        fn resolve(&self, _resolver: Self::Resolver, out: Place<Self::Archived>) {
            out.write(*self);
        }
    }

    impl<const N: usize, S> Serialize<S> for FixedStr<N>
    where
        S: Fallible + ?Sized,
    {
        #[inline]
        fn serialize(&self, _serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            Ok(())
        }
    }

    impl<const N: usize, D> Deserialize<Self, D> for FixedStr<N>
    where
        D: Fallible + ?Sized,
    {
        #[inline]
        fn deserialize(&self, _deserializer: &mut D) -> Result<Self, D::Error> {
            Ok(*self)
        }
    }

    unsafe impl<const N: usize, C> CheckBytes<C> for FixedStr<N>
    where
        C: Fallible + ?Sized,
        C::Error: Source,
    {
        unsafe fn check_bytes<'a>(value: *const Self, _context: &mut C) -> Result<(), C::Error> {
            from_utf8(from_raw_parts(value.cast::<u8>(), N)).into_error()?;

            Ok(())
        }
    }
};

#[cfg(feature = "schema")]
const _: () = {
    use alloc::{borrow::ToOwned, boxed::Box, string::String};

    use schemars::{
        schema::{InstanceType, Metadata, Schema, SchemaObject, SingleOrVec},
        JsonSchema,
    };

    impl<const N: usize> JsonSchema for FixedStr<N> {
        fn schema_name() -> String {
            "FixedStr".to_owned()
        }

        fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> Schema {
            let mut obj = SchemaObject {
                metadata: Some(Box::new(Metadata {
                    description: Some(alloc::format!("FixedStr<{N}>")),
                    ..Default::default()
                })),
                instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
                ..Default::default()
            };

            obj.string().pattern = Some(alloc::format!(r#"[\x00-\x7F]{{{N}}}"#));
            obj.string().min_length = Some(N as u32);
            obj.string().max_length = Some(N as u32);

            Schema::Object(obj)
        }
    }
};
