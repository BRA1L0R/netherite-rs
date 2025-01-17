use bytes::{Buf, BufMut};
use thiserror::Error;

#[derive(Debug, Error)]
/// Defines an error that could be
/// returned while deserializing a VarInt
pub enum VarIntError {
    /// no more bytes could be read from the buffer
    #[error("not enuogh bytes to complete varint")]
    Eof,
    /// encoded varint is bigger than i32
    #[error("varint length exceeds max")]
    Big,
}

trait BitExtension {
    fn add_continue(self) -> Self;
    fn mask_continue(self) -> Self;
    fn is_continue(&self) -> bool;
}

trait Modify: Sized {
    fn modify(&mut self, op: impl FnOnce(Self) -> Self);
}

impl<T: Copy> Modify for T {
    #[inline]
    fn modify(&mut self, op: impl FnOnce(Self) -> Self) {
        *self = op(*self);
    }
}

const CONTINUE_BIT: u8 = 0x80;
const SHIFT: usize = 7;

impl BitExtension for u8 {
    fn add_continue(self) -> Self {
        self | CONTINUE_BIT
    }

    fn mask_continue(self) -> Self {
        self & !CONTINUE_BIT
    }

    fn is_continue(&self) -> bool {
        (self & CONTINUE_BIT) != 0
    }
}

struct Counter<B> {
    buffer: B,
    count: usize,
}

impl<B> Counter<B> {
    fn count(&mut self) -> usize {
        self.count
    }
}

unsafe impl<B: BufMut> BufMut for Counter<B> {
    fn remaining_mut(&self) -> usize {
        self.buffer.remaining_mut()
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.count += cnt;
        self.buffer.advance_mut(cnt)
    }

    fn chunk_mut(&mut self) -> &mut bytes::buf::UninitSlice {
        self.buffer.chunk_mut()
    }
}

trait CounterExt: Sized {
    fn counter(self) -> Counter<Self>;
}

impl<B: BufMut> CounterExt for B {
    fn counter(self) -> Counter<Self> {
        Counter {
            buffer: self,
            count: 0,
        }
    }
}

/// Reads a varint from `buffer`, advancing the internal cursor
///
/// If you don't want the internal cursor to be advanced, as to check
/// for early EOF while decoding, for example, partial buffers, pass an
/// immutable reference to the buffer
///
/// ```no_run
/// # use netherite::varint::read_varint;
///
/// let original = &[0x10];
/// let buffer = &original[..];
///
/// // this one advances the buffer pointer
/// read_varint(buffer);
///
/// // this one does not
/// read_varint(&buffer[..]);
/// ```
pub fn read_varint(mut buffer: impl Buf) -> Result<(usize, i32), VarIntError> {
    let mut buf: u32 = 0;

    for i in 0..5 {
        let byte = buffer
            .has_remaining()
            .then(|| buffer.get_u8())
            .ok_or(VarIntError::Eof)?;

        buf |= (byte.mask_continue() as u32) << (i * SHIFT);

        if !byte.is_continue() {
            return Ok((i + 1, buf as i32));
        }
    }

    Err(VarIntError::Big)
}

/// Encodes `val` in varint format into BufMut
pub fn write(mut writer: impl BufMut, val: i32) -> usize {
    let val = val as u32;

    let mut buf = [0u8; 5];
    let mut counted_buf = buf.counter();

    std::iter::successors(Some(val), |val| Some(val >> SHIFT))
        .map_while(|val| (val != 0).then_some(val as u8))
        .map(BitExtension::add_continue)
        .for_each(|b| counted_buf.put_u8(b));

    let varint_size = std::cmp::max(counted_buf.count(), 1);

    buf[varint_size - 1].modify(BitExtension::mask_continue);
    writer.put_slice(&buf[..varint_size]);

    varint_size
}

/// Cheaply calculates the size of a supposed VarInt
/// containing this i32.
///
/// Runs a logarithm which is optimized to a byte
/// shift iterated at worst 5 times
pub const fn size(val: i32) -> usize {
    let val = val as u32;

    match val.checked_ilog(1 << SHIFT) {
        Some(t) => t as usize + 1,
        None => 1,
    }
}

#[cfg(test)]
mod test {
    use crate::varint::read_varint;

    use super::{size, write};

    const TEST: &[(i32, &[u8])] = &[
        (0, &[0x00]),
        (127, &[0x7f]),
        (255, &[0xff, 0x01]),
        (25565, &[0xdd, 0xc7, 0x01]),
        (2097151, &[0xff, 0xff, 0x7f]),
        (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
        (-2147483648, &[0x80, 0x80, 0x80, 0x80, 0x08]),
        (-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
    ];

    #[test]
    fn varint() {
        for (expected, input) in TEST.iter().copied() {
            let (bytes, res) = read_varint(input).unwrap();

            assert_eq!(bytes, input.len());
            assert_eq!(res, expected);
        }
    }

    #[test]
    fn varint_write() {
        let mut buf = [0; 5];
        for (input, expected) in TEST.iter().copied() {
            let written = write(&mut buf[..], input);

            assert_eq!(written, expected.len());
            assert_eq!(&buf[..written], expected);
        }
    }

    #[test]
    fn varint_size() {
        for (input, expected) in TEST.iter().copied() {
            let size = size(input);

            assert_eq!(size, expected.len())
        }
    }
}
