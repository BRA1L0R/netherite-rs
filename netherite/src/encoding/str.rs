use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use bytes::Bytes;

use crate::{DeError, Deserialize, Serialize};

/// A string type backed by Bytes
///
/// Useful for deserializing from a Bytes,
/// which just increases ref count and doesn't copy
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Str {
    inner: Bytes,
}

impl Str {
    /// Panic:
    /// panics if slice isn't a subset of self
    pub fn slice(&self, slice: &str) -> Self {
        let inner = self.inner.slice_ref(slice.as_bytes());
        Self { inner }
    }

    /// Creates a new Str from a &'static str, without allocating
    pub fn from_static(str: &'static str) -> Self {
        let inner = Bytes::from_static(str.as_bytes());
        Self { inner }
    }
}

impl Deserialize for Str {
    fn deserialize(buffer: impl bytes::Buf) -> Result<Self, crate::DeError> {
        let inner = Bytes::deserialize(buffer)?;

        std::str::from_utf8(&inner)
            .is_ok()
            .then_some(Str { inner })
            .ok_or(DeError::InvalidData)
    }
}

impl Serialize for Str {
    fn serialize(&self, buf: impl bytes::BufMut) {
        self.deref().serialize(buf)
    }

    fn size(&self) -> usize {
        self.deref().size()
    }
}

impl Deref for Str {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // UTF-8 correctness is checked in deserialization
        unsafe { std::str::from_utf8_unchecked(&self.inner) }
    }
}

impl Debug for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.deref(), f)
    }
}

impl PartialEq<str> for Str {
    fn eq(&self, other: &str) -> bool {
        self == other
    }
}
