#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

/// tokio_util codec for serializing and deserializing Minecraft packets
pub mod codec;
/// traits and types for data encoding of Minecraft packets
pub mod encoding;
/// structs representing Minecraft packets
pub mod packet;
/// Minecraft VarInt implementation
pub mod varint;

pub(crate) mod peek;

pub use codec::MinecraftCodec;
pub use encoding::{
    de::{DeError, Deserialize},
    ser::Serialize,
};
pub use netherite_derive::{Deserialize, Serialize};

pub use bytes as _bytes_export;
