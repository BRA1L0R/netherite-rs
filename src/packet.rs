use crate::encoding::serialize_bytes;
use crate::encoding::{deserialize_bytes, packetid::PacketId};
use crate::{DeError, Deserialize, Serialize};
use bytes::Bytes;

#[derive(Debug)]
/// A Minecraft frame unit composing
/// of a packet_id and byte data
pub struct RawPacket {
    /// PacketID
    pub packet_id: i32,
    /// Data
    pub data: Bytes,
}

impl RawPacket {
    /// ```no_run
    /// self.packet_id == T::ID
    /// ```
    pub fn is<T: PacketId>(&self) -> bool {
        self.packet_id == T::ID
    }

    /// Warning: you should check with [`Self::is`] the packet_id
    /// of the data you're deserializing is the same as [`self.packet_id`]
    pub fn deserialize_unchecked<T>(&self) -> Result<T, DeError>
    where
        T: Deserialize + PacketId,
    {
        let buffer = self.data.clone(); // cheap to clone (zero-copy)
        deserialize_bytes(buffer)
    }

    /// If packet_id is the same as T calls [`Self::deserialize_unchecked`]
    pub fn deserialize<T>(&self) -> Option<Result<T, DeError>>
    where
        T: Deserialize + PacketId,
    {
        self.is::<T>().then(|| self.deserialize_unchecked())
    }
}

impl<T: Serialize + PacketId> From<T> for RawPacket {
    fn from(value: T) -> Self {
        RawPacket {
            packet_id: T::ID,
            data: serialize_bytes(value),
        }
    }
}
