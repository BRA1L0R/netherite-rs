use crate::varint::{self, read_varint, write_varint};

use super::{de::Deserialize, ser::Serialize};

/// newtype wrapper that defines
/// a varint-encoded i32
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct VarInt(pub i32);

impl<'de> Deserialize<'de> for VarInt {
    fn deserialize(buffer: &mut super::BorrowedBuffer<'de>) -> Result<Self, super::de::DeError> {
        read_varint(&mut buffer.buf)
            .map(|(_, varint)| Self(varint))
            .map_err(Into::into)
    }
}

impl Serialize for VarInt {
    fn serialize(&self, buf: impl bytes::BufMut) -> Result<(), super::ser::SerError> {
        write_varint(buf, self.0);
        Ok(())
    }

    fn size(&self) -> usize {
        varint::size(self.0)
    }
}
