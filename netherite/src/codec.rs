use std::{cmp::Ordering, io::Write, ops::Deref, usize};

use bytes::{Buf, BufMut, BytesMut};
use flate2::{write::ZlibEncoder, Compression};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

#[cfg(test)]
mod test;

pub mod dual;

use crate::{
    encoding::packetid::PacketId,
    packet::RawPacket,
    peek::PeekBuffer,
    varint::{self, VarIntError},
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

/// kept for backwards compatibility with old naming
pub type MinecraftCodec = UncompressedCodec;

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

        // if there are missing bytes, preemptively reserve
        // space in the buffer to accomodate them
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

pub struct CompressedCodec {
    /// the treshold at which the codec will start zlib compressing the packet data
    compression_threshold: usize,

    /// max packet size the codec is willing to decode from read stream
    max_size: usize,

    // internal reusable buffers
    compressed_buffer: Vec<u8>,
    uncompressed_buffer: Vec<u8>,
}

impl Default for CompressedCodec {
    fn default() -> Self {
        Self {
            compression_threshold: 256,
            max_size: usize::MAX,

            compressed_buffer: vec![],
            uncompressed_buffer: vec![],
        }
    }
}

impl CompressedCodec {
    /// sets the compression threshold.
    ///
    /// The compression threshold indicates the minimum packet size after which
    /// packet data starts getting compressed.
    pub fn compression(self, threshold: usize) -> Self {
        Self {
            compression_threshold: threshold,
            ..self
        }
    }

    /// Maximum size the codec is willing to receive from the connection
    pub fn max_size(self, max_size: usize) -> Self {
        Self { max_size, ..self }
    }

    /// sets the compression treshold for the Codec
    pub fn set_compression(&mut self, treshold: usize) {
        self.compression_threshold = treshold
    }

    /// retrieves the currently set compression treshold
    pub fn compression_treshold(&self) -> usize {
        self.compression_threshold
    }
}

impl Decoder for CompressedCodec {
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

        // if there are missing bytes, preemptively reserve
        // space in the buffer to accomodate them
        if let Some(missing @ 1..) = packet_len.checked_sub(remaining) {
            src.reserve(missing);
            return Ok(None);
        }

        src.advance(packet_len_size);

        let data_length = varint::read_varint(src)?;

        todo!()
    }
}

impl Encoder<&RawPacket> for CompressedCodec {
    type Error = CodecError;

    fn encode(&mut self, item: &RawPacket, mut dst: &mut BytesMut) -> Result<(), Self::Error> {
        let size = varint::size(item.packet_id) + item.data.len();

        debug_assert!(i32::try_from(size).is_ok()); // 👍

        if size >= self.compression_threshold {
            // compressed

            self.compressed_buffer.clear();
            let mut encoder = ZlibEncoder::new(&mut self.compressed_buffer, Compression::default());

            // ce ripassiamo
            let mut varint = [0; 4];
            let varint_written = varint::write(&mut varint[..], item.packet_id);
            encoder.write_all(&varint[..varint_written])?;
            encoder.write_all(&item.data)?;

            let compressed_data = encoder.finish()?;

            // final packet

            let data_length: i32 = size.try_into().map_err(|_| CodecError::Size)?;

            let packet_length = varint::size(data_length) + compressed_data.len();
            let packet_length = packet_length.try_into().map_err(|_| CodecError::Size)?;

            varint::write(&mut dst, packet_length);
            varint::write(&mut dst, data_length);
            dst.extend_from_slice(&self.compressed_buffer) // cazzo extendilo
        } else {
            let data_length = 0i32;

            let packet_length = varint::size(data_length) + item.data.len();
            let packet_length = packet_length.try_into().map_err(|_| CodecError::Size)?;

            varint::write(&mut dst, packet_length);
            varint::write(&mut dst, data_length);
            dst.extend_from_slice(&item.data)
        }
        Ok(())
    }
}

impl<T: Serialize + PacketId> Encoder<T> for CompressedCodec {
    type Error = CodecError;

    fn encode(&mut self, data: T, mut dst: &mut BytesMut) -> Result<(), Self::Error> {
        let size = varint::size(T::ID) + data.size();

        debug_assert!(i32::try_from(size).is_ok()); // 👍

        if size >= self.compression_threshold {
            self.uncompressed_buffer.clear();
            self.compressed_buffer.clear();

            let uncompressed_size = varint::size(T::ID) + data.size();
            let data_length: i32 = uncompressed_size.try_into().map_err(|_| CodecError::Size)?;

            self.uncompressed_buffer.reserve(uncompressed_size);
            varint::write(&mut self.uncompressed_buffer, T::ID);
            data.serialize(&mut self.uncompressed_buffer);

            let mut encoder = ZlibEncoder::new(&mut self.compressed_buffer, Compression::default());
            encoder.write_all(&self.uncompressed_buffer)?;
            encoder.finish()?;

            let packet_size = varint::size(data_length) + self.compressed_buffer.len();
            let packet_size: i32 = packet_size.try_into().map_err(|_| CodecError::Size)?;

            // let packet_size = varint::si
            varint::write(&mut dst, packet_size);
            varint::write(&mut dst, data_length);
            dst.extend_from_slice(&self.compressed_buffer);
        } else {
            let data_length = 0i32;

            let packet_length = varint::size(data_length) + varint::size(T::ID) + data.size();
            let packet_length = packet_length.try_into().map_err(|_| CodecError::Size)?;

            varint::write(&mut dst, packet_length);
            varint::write(&mut dst, data_length);

            varint::write(&mut dst, T::ID);
            data.serialize(dst);
        }

        Ok(())
    }
}
