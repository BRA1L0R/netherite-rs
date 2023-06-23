use bytes::Buf;
use serde::de::SeqAccess;
use std::marker::PhantomData;

use super::VarIntError;

pub trait ConsumeByte {
    fn consume_byte(&mut self) -> Result<u8, VarIntError>;
}

impl<B: Buf> ConsumeByte for B {
    // type Error = VarIntError;
    fn consume_byte(&mut self) -> Result<u8, VarIntError> {
        self.has_remaining()
            .then(|| self.get_u8())
            .ok_or(VarIntError::Eof)
    }
}

pub struct SeqAccessAdapter<'de, S>(S, PhantomData<&'de ()>);

impl<S> SeqAccessAdapter<'_, S> {
    pub fn new(access: S) -> Self {
        Self(access, PhantomData::default())
    }
}

impl<'de, S: SeqAccess<'de>> ConsumeByte for SeqAccessAdapter<'de, S> {
    fn consume_byte(&mut self) -> Result<u8, VarIntError> {
        match self.0.next_element() {
            Ok(Some(v)) => Ok(v),
            _ => Err(VarIntError::Eof),
        }
    }
}
