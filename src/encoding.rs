use bytes::{Bytes, BytesMut};

use self::{
    de::{DeError, Deserialize},
    ser::{SerError, Serialize},
};

pub mod de;
pub mod packetid;
pub mod ser;
pub mod varint;

mod macros;
#[cfg(test)]
pub mod test;

pub struct BorrowedBuffer<'de> {
    // pointer that gets advanced like a cursor
    // as the content is deserialized
    buf: &'de [u8],
}

impl<'de> BorrowedBuffer<'de> {
    pub fn new(buf: &'de [u8]) -> Self {
        Self { buf }
    }

    pub fn from_bytes(buf: &'de Bytes) -> Self {
        use std::ops::Deref;
        Self { buf: buf.deref() }
    }
}

pub fn serialize_bytes<T: Serialize>(serialize: T) -> Result<Bytes, SerError> {
    let prealloc = serialize.size();
    let mut buf = BytesMut::with_capacity(prealloc);

    serialize.serialize(&mut buf)?;
    Ok(buf.freeze())
}

pub fn deserialize_bytes<'de, T: Deserialize<'de>>(buf: &'de [u8]) -> Result<T, DeError> {
    T::deserialize(&mut BorrowedBuffer::new(buf))
}
