use thiserror::Error;

use crate::varint::VarIntError;

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
