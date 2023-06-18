pub mod codec;
#[macro_use]
pub mod encoding;
mod peek;
pub mod varint;

pub use encoding::{
    de::{DeError, Deserialize},
    ser::{SerError, Serialize},
};
pub use netherite_derive::{Deserialize, Serialize};

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
