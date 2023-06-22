use bytes::{Buf, Bytes, BytesMut};

use self::{
    de::{DeError, Deserialize},
    ser::Serialize,
};

/// traits and implementations for deserialization
pub mod de;
/// defines a trait that binds a packet_id to a deserializable type
pub mod packetid;
/// traits and implementations for serialization
pub mod ser;
/// cheaply deserializable and clonable string type
pub mod str;
/// wrapper type around an [`i32`] for serializing and deserializing
/// a Varint
pub mod varint;

mod macros;
#[cfg(test)]
pub mod test;

/// Serializes `data` into Bytes, preallocating a buffer
/// with size `data.size` (for more information read [`Serialize::size`])
pub fn serialize_bytes<T: Serialize>(data: T) -> Bytes {
    let prealloc = data.size();
    let mut buf = BytesMut::with_capacity(prealloc);

    data.serialize(&mut buf);

    buf.freeze()
}

/// Deserializes `buf` into `T` borrowing the data for `'de`
pub fn deserialize_bytes<T: Deserialize>(buf: impl Buf) -> Result<T, DeError> {
    T::deserialize(buf)
}
