use bytes::{Buf, Bytes};
use std::{num::TryFromIntError, str::Utf8Error};
use thiserror::Error;

use crate::varint::{read_varint, VarIntError};

#[derive(Debug, Error)]
/// Defines an error that could be thrown by
/// the deserialization process
pub enum DeError {
    #[error("received data is invalid")]
    /// Deserialized data is invalid. This could mean
    /// the data contains an invalid variant.
    InvalidData,

    /// String type received invalid UTF-8 data
    #[error("utf8: {0}")]
    Utf8(#[from] Utf8Error),

    /// Deserialized structure requires more data
    /// than there's available
    #[error("not enough data to deserialize")]
    Eof,
}

impl From<VarIntError> for DeError {
    fn from(value: VarIntError) -> Self {
        match value {
            VarIntError::Eof => Self::Eof,
            VarIntError::Big => Self::InvalidData,
        }
    }
}

impl From<TryFromIntError> for DeError {
    fn from(_: TryFromIntError) -> Self {
        Self::InvalidData
    }
}

/// Defines a piece of information that can be
/// deserialized from a serialized Minecraft packet
pub trait Deserialize: Sized {
    /// Instantiate a `Self` from a buffer `buffer`
    fn deserialize(buffer: impl Buf) -> Result<Self, DeError>;
}

impl Deserialize for bool {
    fn deserialize(buffer: impl Buf) -> Result<Self, DeError> {
        let a = u8::deserialize(buffer)?;
        match a {
            1 => Ok(true),
            0 => Ok(false),
            _ => Err(DeError::InvalidData),
        }
    }
}

impl Deserialize for Bytes {
    fn deserialize(mut buffer: impl Buf) -> Result<Self, DeError> {
        let (_, length) = read_varint(&mut buffer)?;
        let length = length.try_into().map_err(|_| DeError::InvalidData)?;

        (length <= buffer.remaining())
            .then(|| buffer.copy_to_bytes(length))
            .ok_or(DeError::Eof)
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize(mut buffer: impl Buf) -> Result<Self, DeError> {
        let present = bool::deserialize(&mut buffer)?;

        let res = match present {
            true => Some(T::deserialize(buffer)?),
            false => None,
        };

        Ok(res)
    }
}

impl Deserialize for () {
    fn deserialize(_: impl Buf) -> Result<Self, DeError> {
        Ok(())
    }
}

macro_rules! impl_int {
    ($type:ty, $method:tt) => {
        impl Deserialize for $type {
            fn deserialize(mut buffer: impl Buf) -> Result<Self, DeError> {
                (buffer.remaining() >= std::mem::size_of::<$type>())
                    .then(|| buffer.$method())
                    .ok_or(DeError::Eof)
            }
        }
    };
}

impl_int!(u8, get_u8);
impl_int!(i8, get_i8);
impl_int!(u16, get_u16);
impl_int!(i16, get_i16);
impl_int!(u32, get_u32);
impl_int!(i32, get_i32);
impl_int!(u64, get_u64);
impl_int!(i64, get_i64);
