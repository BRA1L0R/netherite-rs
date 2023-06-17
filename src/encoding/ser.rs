use bytes::BufMut;
use thiserror::Error;

use crate::varint;

#[derive(Debug, Error)]
pub enum SerError {
    #[error("tried encoding a value bigger than its encoded limit")]
    Size,
}

pub trait Serialize {
    /// serializes &self into an impl BufMut
    fn serialize(&self, buf: impl BufMut) -> Result<(), SerError>;

    /// size in bytes, allows for preallocation
    /// of serialization buffer
    fn size(&self) -> usize;
}

impl<T: Serialize> Serialize for &T {
    fn serialize(&self, buf: impl BufMut) -> Result<(), SerError> {
        (*self).serialize(buf)
    }

    fn size(&self) -> usize {
        (*self).size()
    }
}

impl Serialize for &str {
    fn serialize(&self, mut buf: impl BufMut) -> Result<(), SerError> {
        let size = self.len().try_into().map_err(|_| SerError::Size)?;
        varint::write_varint(&mut buf, size);

        buf.put_slice(self.as_bytes());
        Ok(())
    }

    fn size(&self) -> usize {
        let size = self.len().try_into().unwrap_or(i32::MAX); // will get caught by serialize impl
        varint::size(size) + self.len()
    }
}

impl Serialize for String {
    fn serialize(&self, buf: impl BufMut) -> Result<(), SerError> {
        self.as_str().serialize(buf)
    }

    fn size(&self) -> usize {
        self.as_str().size()
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize(&self, mut buf: impl BufMut) -> Result<(), SerError> {
        self.is_some().serialize(&mut buf)?;
        if let Some(val) = self {
            val.serialize(buf)?;
        }

        Ok(())
    }

    fn size(&self) -> usize {
        1 + self.as_ref().map(|val| val.size()).unwrap_or_default()
    }
}

impl Serialize for () {
    fn serialize(&self, _: impl BufMut) -> Result<(), SerError> {
        Ok(())
    }

    fn size(&self) -> usize {
        0
    }
}

macro_rules! impl_int {
    ($type:ty, $method:tt) => {
        impl Serialize for $type {
            fn serialize(&self, mut buf: impl BufMut) -> Result<(), SerError> {
                buf.$method(*self as _);
                Ok(())
            }

            fn size(&self) -> usize {
                std::mem::size_of::<$type>()
            }
        }
    };
}

impl_int!(u8, put_u8);
impl_int!(i8, put_i8);
impl_int!(u16, put_u16);
impl_int!(i16, put_i16);
impl_int!(u32, put_u32);
impl_int!(i32, put_i32);
impl_int!(u64, put_u64);
impl_int!(i64, put_i64);
impl_int!(bool, put_u8);
