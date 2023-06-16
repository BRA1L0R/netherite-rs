use crate::varint::read_varint;

use super::Error;

use bytes::Buf;
use serde::{de::SeqAccess, Deserialize, Deserializer};

macro_rules! check_remaining {
    ($buf:expr, $type:ty) => {
        $buf.remaining()
            .checked_sub(std::mem::size_of::<$type>())
            .ok_or(Error::Eof)?;
    };
}

pub fn deserialize_bytes<'a, T: Deserialize<'a>>(slice: &'a [u8]) -> Result<T, Error> {
    let mut de = MinecraftDeserializer { inner: slice };
    T::deserialize(&mut de)
}

pub struct MinecraftDeserializer<'a> {
    inner: &'a [u8],
}

impl<'de, 'a: 'de> Deserializer<'de> for &'_ mut MinecraftDeserializer<'a> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::TypeSpec)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let bool = match self.inner.get_u8() {
            0 => false,
            1 => true,
            _ => return Err(Error::Variant),
        };

        visitor.visit_bool(bool)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        check_remaining!(self.inner, i8);
        visitor.visit_i8(self.inner.get_i8())
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        check_remaining!(self.inner, i16);
        visitor.visit_i16(self.inner.get_i16())
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        check_remaining!(self.inner, i32);
        visitor.visit_i32(self.inner.get_i32())
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        check_remaining!(self.inner, i64);
        visitor.visit_i64(self.inner.get_i64())
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        check_remaining!(self.inner, u8);
        visitor.visit_u8(self.inner.get_u8())
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        check_remaining!(self.inner, u16);
        visitor.visit_u16(self.inner.get_u16())
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        check_remaining!(self.inner, u32);
        visitor.visit_u32(self.inner.get_u32())
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        check_remaining!(self.inner, u64);
        visitor.visit_u64(self.inner.get_u64())
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        check_remaining!(self.inner, f32);
        visitor.visit_f32(self.inner.get_f32())
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        check_remaining!(self.inner, f64);
        visitor.visit_f64(self.inner.get_f64())
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // TODO: unify impl with deserialize_bytes

        let (_, str_size) = read_varint(&mut self.inner)?;
        let str_size = str_size.try_into()?;

        let decoded = self.inner.get(..str_size).ok_or(Error::Eof)?;
        let str = std::str::from_utf8(decoded)?;
        self.inner.advance(str_size);

        visitor.visit_borrowed_str(str)

        // self.deserialize_bytes(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // we don't own the data, sorry
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let (_, size) = read_varint(&mut self.inner)?;
        let size = size.try_into()?;

        let bytes = self.inner.get(..size).ok_or(Error::Size)?;
        self.inner.advance(size);

        visitor.visit_borrowed_bytes(bytes)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match bool::deserialize(&mut *self)? {
            true => visitor.visit_some(self),
            false => visitor.visit_none(),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let (_, length) = read_varint(&mut self.inner)?;
        let length: usize = length.try_into()?;

        let seq = MinecraftSeq::new(self, length);
        visitor.visit_seq(seq)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let seq = MinecraftSeq::new(self, len);
        visitor.visit_seq(seq)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unimplemented)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unimplemented)
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unimplemented)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Error::Unimplemented)
    }
}

pub struct MinecraftSeq<'deref, 'b> {
    de: &'deref mut MinecraftDeserializer<'b>,
    remaining: usize,
}

impl<'deref, 'data> MinecraftSeq<'deref, 'data> {
    pub fn new(de: &'deref mut MinecraftDeserializer<'data>, remaining: usize) -> Self {
        Self { de, remaining }
    }
}

impl<'deref, 'de: 'deref, 'data: 'de> SeqAccess<'de> for MinecraftSeq<'deref, 'data> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let Some(remaining) = self.remaining.checked_sub(1) else { return Ok(None) };
        self.remaining = remaining;

        seed.deserialize(&mut *self.de).map(Some)
    }
}
