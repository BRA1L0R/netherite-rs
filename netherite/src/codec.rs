use bytes::BytesMut;

use crate::{codec::error::CodecError, packet::RawPacket, varint};

#[cfg(test)]
mod test;

// pub mod dual;
pub mod compressed;
pub mod error;
pub mod uncompressed;

// kept for backwards compatibility with old naming
// pub type MinecraftCodec = UncompressedCodec;

fn decode_raw_packet(
    mut buf: &mut BytesMut,
    payload_length: usize,
) -> Result<RawPacket, CodecError> {
    let (packet_id_size, packet_id) = varint::read_varint(&mut buf)?;

    let data_length = payload_length
        .checked_sub(packet_id_size)
        .ok_or(CodecError::Size)?;

    let data = buf.split_off(data_length).freeze();
    Ok(RawPacket { packet_id, data })
}
