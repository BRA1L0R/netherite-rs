mod codec;
mod encoding;
pub mod varint;
pub mod peek;

use codec::MinecraftCodec;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

struct Connection {
    inner: Framed<TcpStream, MinecraftCodec>,
}

impl Connection {
    fn new(stream: TcpStream) -> Self {
        let inner = Framed::new(stream, MinecraftCodec::default());
        Self { inner }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use futures::{Stream, StreamExt};
    use tokio_util::codec::FramedRead;

    use crate::{
        codec::{CodecError, MinecraftPacket},
        MinecraftCodec,
    };

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
