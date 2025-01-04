use bytes::Buf;

use crate::varint::{self, read_varint, write};

use super::{de::Deserialize, ser::Serialize};

/// newtype wrapper that defines
/// a varint-encoded i32
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct VarInt(pub i32);

impl Deserialize for VarInt {
    fn deserialize(buffer: impl Buf) -> Result<Self, super::de::DeError> {
        read_varint(buffer)
            .map(|(_, varint)| Self(varint))
            .map_err(Into::into)
    }
}

impl Serialize for VarInt {
    fn serialize(&self, buf: impl bytes::BufMut) {
        write(buf, self.0);
    }

    fn size(&self) -> usize {
        varint::size(self.0)
    }
}
