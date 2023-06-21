use std::ops::Deref;
use std::pin::Pin;

use crate::encoding::{deserialize_bytes, packetid::PacketId, serialize_bytes};
use crate::{DeError, Deserialize, Serialize};
use bytes::Bytes;

#[derive(Debug)]
pub struct RawPacket {
    pub packet_id: i32,
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
    pub fn deserialize_unchecked<T>(&self) -> Result<Packet<T>, DeError>
    where
        T: Deserialize + PacketId,
    {
        let buffer = self.data.clone(); // cheap to clone (zero-copy)
        deserialize_bytes(buffer).map(Packet)
    }

    /// If packet_id is the same as T calls [`Self::deserialize_unchecked`]
    pub fn deserialize<T>(&self) -> Option<Result<Packet<T>, DeError>>
    where
        T: Deserialize + PacketId,
    {
        self.is::<T>().then(|| self.deserialize_unchecked())
    }
}

/// Utility wrapper for sending unserialized packets
/// over a Codec, or pre-serialization through [`Self::pack`]
pub struct Packet<T: PacketId>(pub T);

impl<T: PacketId> Packet<T> {
    pub fn new(packet: T) -> Self {
        Self(packet)
    }

    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: PacketId + Serialize> Packet<T> {
    /// Packs a Packet into a RawPacket
    ///
    /// Note: codec support efficient serialization of [`Packet`]
    /// without passing through [`RawPacket`]
    pub fn pack(&self) -> RawPacket {
        RawPacket {
            packet_id: T::ID,
            data: serialize_bytes(&self.0),
        }
    }
}

impl<T: PacketId> From<T> for Packet<T> {
    fn from(value: T) -> Self {
        Packet(value)
    }
}

#[cfg(test)]
mod test {
    // use super::MyStruct;

    #[test]
    fn test() {
        // MyStruct { }
    }
}
