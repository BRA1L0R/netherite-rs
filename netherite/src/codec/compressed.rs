use bytes::{Buf, BufMut, BytesMut};
use flate2::{
    Compression,
    write::{ZlibDecoder, ZlibEncoder},
};
use std::io::Write;
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    Serialize,
    codec::error::CodecError,
    encoding::packetid::PacketId,
    packet::RawPacket,
    peek::PeekBuffer,
    varint::{self, VarIntError},
};

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

        let (data_len_size, data_len) = varint::read_varint(&mut src)?;
        let data_len = data_len.try_into().map_err(|_| CodecError::Size)?;
        if !(1..self.max_size).contains(&data_len) {
            return Err(CodecError::Size);
        }

        // subtract data_len to get the length of the rest of the packet
        let payload_length = packet_len
            .checked_sub(data_len_size)
            .ok_or(CodecError::Size)?;

        // data_len = 0 => uncompressed
        // data_len > 0 => compressed, needs zlib decompression
        let packet = match data_len {
            0 => super::decode_raw_packet(src, payload_length)?,
            _ => {
                let mut data = BytesMut::with_capacity(data_len);
                let mut decoder = ZlibDecoder::new((&mut data).writer());

                decoder.write_all(&src[..payload_length])?;
                src.advance(payload_length);
                decoder.finish()?;

                super::decode_raw_packet(&mut data, data_len)?
            }
        };

        Ok(Some(packet))
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
            let mut varint = [0; 5];
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

            // final packet assembly
            //
            varint::write(&mut dst, packet_length);
            varint::write(&mut dst, data_length);
            varint::write(&mut dst, T::ID);
            data.serialize(dst);
        }

        Ok(())
    }
}
