use std::io::Cursor;

use futures::{Stream, StreamExt};
use tokio_util::codec::FramedRead;

use super::UncompressedCodec;
use crate::codec::{CodecError, RawPacket};

macro_rules! block {
    ($expr:expr) => {
        futures::executor::block_on($expr)
    };
}

fn setup_reader<T>(bytes: T) -> impl Stream<Item = Result<RawPacket, CodecError>>
where
    T: AsRef<[u8]> + Unpin,
{
    FramedRead::new(Cursor::new(bytes), UncompressedCodec::default())
}

// TODO
// fn setup_writer<T>(bytes: impl AsyncWrite) -> impl Sink<RawPacket> {
//     FramedWrite::new(bytes, MinecraftCodec::default())
// }

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

// Get derive macros to work within crate
mod netherite {
    pub use crate::*;
}
