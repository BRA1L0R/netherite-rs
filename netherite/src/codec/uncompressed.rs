use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    Serialize,
    codec::error::CodecError,
    encoding::packetid::PacketId,
    packet::RawPacket,
    peek::PeekBuffer,
    varint::{self, VarIntError},
};

/// Codec for uncompressed and unencrypted
/// Minecraft packets
pub struct UncompressedCodec {
    max_size: usize,
}

impl UncompressedCodec {
    /// sets the maximum size of the packet the decoder is willing
    /// to read from the stream
    pub fn max_size(self, max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Default for UncompressedCodec {
    fn default() -> Self {
        // will get limited by max varint size
        Self {
            max_size: usize::MAX,
        }
    }
}

impl Decoder for UncompressedCodec {
    type Item = RawPacket;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
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

        // if there are missing bytes, preemptively reserve
        // space in the buffer to accomodate them
        if let Some(missing @ 1..) = packet_len.checked_sub(remaining) {
            src.reserve(missing);
            return Ok(None);
        }

        src.advance(packet_len_size);

        let packet = super::decode_raw_packet(src, packet_len)?;

        Ok(Some(packet))
    }
}

impl Encoder<&RawPacket> for UncompressedCodec {
    type Error = CodecError;

    fn encode(&mut self, item: &RawPacket, mut dst: &mut BytesMut) -> Result<(), Self::Error> {
        let packet_size = varint::size(item.packet_id) + item.data.len();
        if packet_size > self.max_size {
            return Err(CodecError::Size);
        }

        let packet_size = packet_size.try_into().map_err(|_| CodecError::Size)?;

        varint::write(&mut dst, packet_size);
        varint::write(&mut dst, item.packet_id);
        dst.put_slice(&item.data);

        Ok(())
    }
}

impl<T: Serialize + PacketId> Encoder<T> for UncompressedCodec {
    type Error = CodecError;

    fn encode(&mut self, data: T, mut dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data_size = varint::size(T::ID) + data.size();
        let data_size: i32 = data_size.try_into().map_err(|_| CodecError::Size)?;

        varint::write(&mut dst, data_size);
        varint::write(&mut dst, T::ID);
        data.serialize(dst);

        Ok(())
    }
}
