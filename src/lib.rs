#![warn(missing_docs)]

pub mod codec;
pub mod encoding;
pub mod varint;

pub(crate) mod peek;
pub mod packet;


pub use encoding::{
    de::{DeError, Deserialize},
    ser::Serialize,
};
pub use netherite_derive::{Deserialize, Serialize};
