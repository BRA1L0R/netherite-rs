use bytes::{Buf, BytesMut};

pub trait PeekBuffer {
    type Peek<'a>: Buf
    where
        Self: 'a;

    // creates a peek adapter that allows for
    // reading without consuming the internal buf
    fn peek(&self) -> Self::Peek<'_>;
}

impl PeekBuffer for BytesMut {
    type Peek<'a> = &'a [u8] where Self: 'a;

    fn peek(&self) -> Self::Peek<'_> {
        &self[..]
    }
}

impl PeekBuffer for &[u8] {
    type Peek<'a> = &'a [u8] where Self: 'a;

    fn peek(&self) -> Self::Peek<'_> {
        self
    }
}
