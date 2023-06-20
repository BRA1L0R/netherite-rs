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
    pub fn deserialize_owned<T>(&self) -> Result<OwnedPacket<T>, DeError>
    where
        for<'de> T: Deserialize<'de> + PacketId,
    {
        deserialize_bytes(&self.data).map(OwnedPacket)
    }
}

pub struct OwnedPacket<T: PacketId>(pub T);

impl<T: PacketId> OwnedPacket<T> {
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

impl<T: PacketId + Serialize> OwnedPacket<T> {
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

impl<T: PacketId> From<T> for OwnedPacket<T> {
    fn from(value: T) -> Self {
        OwnedPacket(value)
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
