use bytes::{Buf, BufMut, BytesMut};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

#[cfg(test)]
mod test;

use crate::{
    encoding::packetid::PacketId,
    packet::RawPacket,
    peek::PeekBuffer,
    varint::{self, write_varint, VarIntError},
    Serialize,
};

#[derive(Debug, Error)]
/// Defines an error that could be thrown off by
/// a Codec
pub enum CodecError {
    /// Underlying I/O returned an error
    #[error("underlying io: {0}")]
    Io(#[from] std::io::Error),

    /// Error deserializing a VarInt
    /// from the Frame
    #[error("varint: {0}")]
    Varint(#[from] VarIntError),

    /// Packet is either too big or too small
    #[error("packet has invalid size")]
    Size,
}

/// Codec for uncompressed and unencrypted
/// Minecraft packets
pub struct MinecraftCodec {
    max_size: usize,
}

impl Default for MinecraftCodec {
    fn default() -> Self {
        // will get limited by max varint size
        Self {
            max_size: usize::MAX,
        }
    }
}

impl Decoder for MinecraftCodec {
    type Item = RawPacket;
    type Error = CodecError;

    fn decode(&mut self, mut src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let (packet_len_size, packet_len) = match varint::read_varint(src.peek()) {
            Err(VarIntError::Eof) => return Ok(None),
            r => r?,
        };

        let packet_len = packet_len.try_into().map_err(|_| CodecError::Size)?;
        if !(1..self.max_size).contains(&packet_len) {
            return Err(CodecError::Size);
        }

        let remaining = src
            .remaining()
            .checked_sub(packet_len_size)
            .expect("buffer should still have varint bytes");

        if let Some(missing @ 1..) = packet_len.checked_sub(remaining) {
            src.reserve(missing);
            return Ok(None);
        }

        src.advance(packet_len_size);

        let (id_len, packet_id) = varint::read_varint(&mut src)?;

        let data_size = packet_len.checked_sub(id_len).ok_or(CodecError::Size)?;
        let data = src.copy_to_bytes(data_size);

        Ok(Some(RawPacket { packet_id, data }))
    }
}

impl Encoder<&RawPacket> for MinecraftCodec {
    type Error = CodecError;

    fn encode(&mut self, item: &RawPacket, mut dst: &mut BytesMut) -> Result<(), Self::Error> {
        let packet_size = varint::size(item.packet_id) + item.data.len();
        if packet_size > self.max_size {
            return Err(CodecError::Size);
        }

        let packet_size = packet_size.try_into().map_err(|_| CodecError::Size)?;

        write_varint(&mut dst, packet_size);
        write_varint(&mut dst, item.packet_id);
        dst.put_slice(&item.data);

        Ok(())
    }
}

impl<T: Serialize + PacketId> Encoder<T> for MinecraftCodec {
    type Error = CodecError;

    fn encode(&mut self, data: T, mut dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data_size = varint::size(T::ID) + data.size();
        let data_size: i32 = data_size.try_into().map_err(|_| CodecError::Size)?;

        write_varint(&mut dst, data_size);
        write_varint(&mut dst, T::ID);
        data.serialize(dst);

        Ok(())
    }
}
