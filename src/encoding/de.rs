use bytes::Buf;
use std::{num::TryFromIntError, str::Utf8Error};
use thiserror::Error;

use crate::varint::{read_varint, VarIntError};

use super::BorrowedBuffer;

#[derive(Debug, Error)]
pub enum DeError {
    #[error("received data is invalid")]
    InvalidData,

    #[error("utf8: {0}")]
    Utf8(#[from] Utf8Error),

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
pub trait Deserialize<'de>: Sized {
    fn deserialize(buffer: &mut BorrowedBuffer<'de>) -> Result<Self, DeError>;
}

impl<'de> Deserialize<'de> for bool {
    fn deserialize(buffer: &mut BorrowedBuffer<'de>) -> Result<Self, DeError> {
        let a = u8::deserialize(buffer)?;
        match a {
            1 => Ok(true),
            0 => Ok(false),
            _ => Err(DeError::InvalidData),
        }
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for &'a [u8] {
    fn deserialize(buffer: &mut BorrowedBuffer<'de>) -> Result<Self, DeError> {
        let (_, size) = read_varint(&mut buffer.buf)?;
        let size: usize = size.try_into()?;

        let slice = buffer.buf.get(..size).ok_or(DeError::Eof)?;
        // let str = std::str::from_utf8(str)?;

        buffer.buf.advance(size);

        Ok(slice)
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for &'de str {
    fn deserialize(buffer: &mut BorrowedBuffer<'de>) -> Result<Self, DeError> {
        let slice = <&[u8]>::deserialize(buffer)?;
        std::str::from_utf8(slice).map_err(Into::into)
    }
}

impl<'de> Deserialize<'de> for String {
    fn deserialize(buffer: &mut BorrowedBuffer<'de>) -> Result<Self, DeError> {
        <&str>::deserialize(buffer).map(ToOwned::to_owned)
    }
}

impl<'de> Deserialize<'de> for Vec<u8> {
    fn deserialize(buffer: &mut BorrowedBuffer<'de>) -> Result<Self, DeError> {
        <&[u8]>::deserialize(buffer).map(ToOwned::to_owned)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Option<T> {
    fn deserialize(buffer: &mut BorrowedBuffer<'de>) -> Result<Self, DeError> {
        let present = bool::deserialize(buffer)?;

        let res = match present {
            true => Some(T::deserialize(buffer)?),
            false => None,
        };

        Ok(res)
    }
}

impl<'de> Deserialize<'de> for () {
    fn deserialize(_: &mut BorrowedBuffer<'de>) -> Result<Self, DeError> {
        Ok(())
    }
}

macro_rules! impl_int {
    ($type:ty, $method:tt) => {
        impl<'de> Deserialize<'de> for $type {
            fn deserialize(buffer: &mut BorrowedBuffer<'de>) -> Result<Self, DeError> {
                (buffer.buf.remaining() >= std::mem::size_of::<$type>())
                    .then(|| buffer.buf.$method())
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
