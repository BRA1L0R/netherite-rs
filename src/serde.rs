use std::{num::TryFromIntError, str::Utf8Error};

use thiserror::Error;

use crate::varint::VarIntError;

pub mod de;
pub mod ser;
pub mod types;

#[cfg(test)]
mod test;

#[derive(Debug, Error)]
pub enum Error {
    #[error("deserializer must know deserialized type upfront, cannot invoke deserialize_any")]
    TypeSpec,

    #[error("there is no minecraft representation for this type")]
    Unimplemented,

    #[error("string or sequence size exceeds varint dimensions")]
    Size,

    #[error("sequence length must be known upfront")]
    SeqLen,

    #[error("invalid type variant")]
    Variant,

    #[error("structure needs more bytes to deserialize")]
    Eof,

    #[error("error deserializing varint: {0}")]
    VarInt(#[from] VarIntError),

    #[error("utf-8 error: {0}")]
    Utf8(#[from] Utf8Error),

    #[error("{0}")]
    Custom(String),
}

impl From<TryFromIntError> for Error {
    fn from(_: TryFromIntError) -> Self {
        Self::Size
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: core::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: core::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}
