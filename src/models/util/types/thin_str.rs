// This file is dual-licensed under either the MIT or Apache 2.0 license.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// You may choose either license to govern your use of this file.
// Any types re-exported from this file also fall under the same license.

//! String type built around [`ThinVec`], so it only uses 8 bytes of stack space.

use alloc::borrow::Cow;
use core::borrow::{Borrow, BorrowMut};
use core::{error, fmt, ops};
use thin_vec::ThinVec;

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use = "Not using the ThinString is wasteful"]
#[repr(transparent)]
pub struct ThinString(ThinVec<u8>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FromUtf8Error {
    bytes: ThinVec<u8>,
    error: core::str::Utf8Error,
}

impl FromUtf8Error {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn into_bytes(self) -> ThinVec<u8> {
        self.bytes
    }

    #[must_use]
    pub fn utf8_error(&self) -> &core::str::Utf8Error {
        &self.error
    }
}

impl fmt::Display for FromUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}

impl error::Error for FromUtf8Error {}

impl fmt::Display for ThinString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl fmt::Debug for ThinString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl ThinString {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(ThinVec::with_capacity(capacity))
    }

    pub fn from_utf8(vec: ThinVec<u8>) -> Result<Self, FromUtf8Error> {
        match core::str::from_utf8(&vec) {
            Ok(_) => Ok(Self(vec)),
            Err(error) => Err(FromUtf8Error { bytes: vec, error }),
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn push_str(&mut self, s: &str) {
        self.0.extend_from_slice(s.as_bytes());
    }

    pub fn push(&mut self, c: char) {
        match c.len_utf8() {
            1 => self.0.push(c as u8),
            _ => self.0.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    #[must_use]
    pub fn into_bytes(self) -> ThinVec<u8> {
        self.0
    }

    pub fn as_mut_vec(&mut self) -> &mut ThinVec<u8> {
        &mut self.0
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for ThinString {
    #[inline(always)]
    fn from(s: &str) -> Self {
        Self(ThinVec::from(s.as_bytes()))
    }
}

impl From<&mut str> for ThinString {
    #[inline(always)]
    fn from(s: &mut str) -> Self {
        Self(ThinVec::from(s.as_bytes()))
    }
}

impl<'a> From<Cow<'a, str>> for ThinString {
    #[inline(always)]
    fn from(s: Cow<'a, str>) -> Self {
        match s {
            Cow::Borrowed(s) => ThinString::from(s),
            Cow::Owned(s) => ThinString::from(s),
        }
    }
}

impl From<String> for ThinString {
    #[inline(always)]
    fn from(s: String) -> Self {
        Self(ThinVec::from(s.into_bytes()))
    }
}

impl From<&String> for ThinString {
    #[inline(always)]
    fn from(s: &String) -> Self {
        ThinString::from(s.as_str())
    }
}

impl From<Box<str>> for ThinString {
    #[inline(always)]
    fn from(s: Box<str>) -> Self {
        Self(ThinVec::from(<Box<[u8]>>::from(s)))
    }
}

impl From<char> for ThinString {
    #[inline(always)]
    fn from(c: char) -> Self {
        let mut thin_string = ThinString::new();
        thin_string.push(c);
        thin_string
    }
}

impl ops::Deref for ThinString {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.0) }
    }
}

impl ops::DerefMut for ThinString {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut str {
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.0) }
    }
}

impl AsRef<str> for ThinString {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsMut<str> for ThinString {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

impl AsRef<[u8]> for ThinString {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<str> for ThinString {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self
    }
}

impl BorrowMut<str> for ThinString {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut str {
        self
    }
}

const _: () = {
    use serde::{de, ser};

    impl ser::Serialize for ThinString {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }

    impl<'de> de::Deserialize<'de> for ThinString {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            struct ThinStringVisitor;

            impl de::Visitor<'_> for ThinStringVisitor {
                type Value = ThinString;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a string")
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(ThinString::from(value))
                }
            }

            deserializer.deserialize_str(ThinStringVisitor)
        }
    }
};

#[cfg(feature = "ts")]
const _: () = {
    use ts_bindgen::{TypeRegistry, TypeScriptDef, TypeScriptType};

    impl TypeScriptDef for ThinString {
        fn register(_: &mut TypeRegistry) -> TypeScriptType {
            TypeScriptType::String(None)
        }
    }
};

#[cfg(feature = "rkyv")]
const _: () = {
    use rkyv::{
        rancor::{Fallible, Source},
        string::{ArchivedString, StringResolver},
        Archive, Deserialize, DeserializeUnsized, Place, Serialize, SerializeUnsized,
    };

    impl Archive for ThinString {
        type Archived = ArchivedString;
        type Resolver = StringResolver;

        fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
            ArchivedString::resolve_from_str(self, resolver, out)
        }
    }

    impl<S: Fallible + ?Sized> Serialize<S> for ThinString
    where
        S::Error: Source,
        str: SerializeUnsized<S>,
    {
        fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            ArchivedString::serialize_from_str(self, serializer)
        }
    }

    impl<D: Fallible + ?Sized> Deserialize<ThinString, D> for ArchivedString
    where
        str: DeserializeUnsized<str, D>,
    {
        fn deserialize(&self, _: &mut D) -> Result<ThinString, D::Error> {
            Ok(ThinString::from(self.as_str()))
        }
    }
};

#[cfg(feature = "pg")]
const _: () = {
    use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};

    impl ToSql for ThinString {
        fn to_sql(&self, ty: &Type, out: &mut bytes::BytesMut) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>>
        where
            Self: Sized,
        {
            self.as_str().to_sql(ty, out)
        }

        fn accepts(ty: &Type) -> bool
        where
            Self: Sized,
        {
            <String as ToSql>::accepts(ty)
        }

        to_sql_checked!();
    }

    impl<'a> FromSql<'a> for ThinString {
        fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            <&str as FromSql>::from_sql(ty, raw).map(ThinString::from)
        }

        fn accepts(ty: &Type) -> bool {
            <String as FromSql>::accepts(ty)
        }
    }
};

impl fmt::Write for ThinString {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl<'a> Extend<&'a str> for ThinString {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(s));
    }
}

impl Extend<Box<str>> for ThinString {
    fn extend<I: IntoIterator<Item = Box<str>>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl Extend<String> for ThinString {
    fn extend<I: IntoIterator<Item = String>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl Extend<ThinString> for ThinString {
    fn extend<I: IntoIterator<Item = ThinString>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl<'a> Extend<Cow<'a, str>> for ThinString {
    fn extend<I: IntoIterator<Item = Cow<'a, str>>>(&mut self, iter: I) {
        iter.into_iter().for_each(move |s: Cow<'a, str>| self.push_str(&s));
    }
}

impl Extend<char> for ThinString {
    fn extend<I: IntoIterator<Item = char>>(&mut self, iter: I) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        iterator.for_each(move |c| self.push(c));
    }
}

impl<'a> Extend<&'a char> for ThinString {
    fn extend<I: IntoIterator<Item = &'a char>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl FromIterator<char> for ThinString {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut thin_string = ThinString::new();
        thin_string.extend(iter);
        thin_string
    }
}

impl<'a> FromIterator<&'a char> for ThinString {
    fn from_iter<I: IntoIterator<Item = &'a char>>(iter: I) -> Self {
        let mut thin_string = ThinString::new();
        thin_string.extend(iter.into_iter().cloned());
        thin_string
    }
}

impl FromIterator<String> for ThinString {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        let mut thin_string = ThinString::new();
        thin_string.extend(iter);
        thin_string
    }
}

impl FromIterator<ThinString> for ThinString {
    fn from_iter<I: IntoIterator<Item = ThinString>>(iter: I) -> Self {
        let mut iterator = iter.into_iter();

        // reuse the first string if possible
        match iterator.next() {
            Some(mut thin_string) => {
                thin_string.extend(iterator);
                thin_string
            }
            None => ThinString::new(),
        }
    }
}

impl<'a> FromIterator<&'a str> for ThinString {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        let mut thin_string = ThinString::new();
        thin_string.extend(iter);
        thin_string
    }
}

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }

        #[allow(unused_lifetimes)]
        impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }
    };
}

impl_eq!(ThinString, str);
impl_eq!(ThinString, &'a str);
impl_eq!(ThinString, String);
impl_eq!(ThinString, Cow<'a, str>);
