mod varint;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};
use varint::{write_varint, VarIntError};

#[derive(Debug, Error)]
enum CodecError {
    #[error("underlying io: {0}")]
    Io(#[from] std::io::Error),

    #[error("varint: {0}")]
    Varint(#[from] VarIntError),

    #[error("packet has invalid size")]
    Size,
}

#[derive(Debug)]
struct MinecraftPacket {
    packet_id: i32,
    data: Bytes,
}

struct MinecraftCodec {
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
    type Item = MinecraftPacket;
    type Error = CodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let (packet_len_size, packet_len) = match varint::read_varint(src) {
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

        let (id_len, packet_id) = varint::read_varint(src)?;
        src.advance(id_len);

        let data_size = packet_len.checked_sub(id_len).ok_or(CodecError::Size)?;
        let data = src.copy_to_bytes(data_size);

        Ok(Some(MinecraftPacket { packet_id, data }))
    }
}

impl Encoder<MinecraftPacket> for MinecraftCodec {
    type Error = CodecError;

    fn encode(&mut self, item: MinecraftPacket, mut dst: &mut BytesMut) -> Result<(), Self::Error> {
        let packet_size = varint::size(item.packet_id) + item.data.len();
        write_varint(
            &mut dst,
            packet_size.try_into().map_err(|_| CodecError::Size)?,
        );

        write_varint(&mut dst, item.packet_id);
        dst.put(item.data);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use futures::{Stream, StreamExt};
    use tokio_util::codec::FramedRead;

    use crate::{CodecError, MinecraftCodec, MinecraftPacket};

    macro_rules! block {
        ($expr:expr) => {
            futures::executor::block_on($expr)
        };
    }

    fn setup_reader<T>(bytes: T) -> impl Stream<Item = Result<MinecraftPacket, CodecError>>
    where
        T: AsRef<[u8]> + Unpin,
    {
        FramedRead::new(Cursor::new(bytes), MinecraftCodec::default())
    }

    #[test]
    fn invalid_size() {
        let mut reader = setup_reader([0]);
        let res = block!(reader.next()).unwrap();

        assert!(matches!(res, Err(CodecError::Size)))
    }

    #[test]
    fn invalid_packetid() {
        let mut reader = setup_reader([0x02, 0xFF, 0xFF]);
        let res = block!(reader.next()).unwrap();

        assert!(matches!(res, Err(CodecError::Varint(_))))
    }

    #[test]
    fn negative_size() {
        let mut reader = setup_reader([0xff, 0xff, 0xff, 0xff, 0x0f]);
        let res = block!(reader.next()).unwrap();

        assert!(matches!(res, Err(CodecError::Size)))
    }

    #[test]
    fn longer_packetid() {
        let mut reader = setup_reader([0x01, 0xff, 0x01]);
        let res = block!(reader.next()).unwrap();

        assert!(matches!(res, Err(CodecError::Size)))
    }

    #[test]
    fn zero_data() {
        let mut reader = setup_reader([0x02, 0xff, 0x01]);
        let res = block!(reader.next()).unwrap();

        let packet = res.unwrap();

        assert_eq!(packet.data.len(), 0);
    }
}
