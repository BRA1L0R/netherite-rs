use serde::{de::Visitor, ser::SerializeTuple};

use crate::varint::{adapters::SeqAccessAdapter, read_varint, write_varint};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct VarInt(pub i32);

impl serde::Serialize for VarInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut buf = [0u8; 5];
        let written = write_varint(&mut buf[..], self.0);

        let mut tup = serializer.serialize_tuple(written)?;
        for b in &buf[..written] {
            tup.serialize_element(b)?;
        }

        tup.end()
    }
}

struct VarIntVisitor;
impl<'de> Visitor<'de> for VarIntVisitor {
    type Value = VarInt;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a newtype varint")
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        use serde::de::Error;

        read_varint(SeqAccessAdapter::new(seq))
            .map_err(|err| Error::custom(format_args!("{err}")))
            .map(|(_, varint)| VarInt(varint))
    }
}

impl<'de> serde::Deserialize<'de> for VarInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(5, VarIntVisitor)
    }
}
