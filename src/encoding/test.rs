use crate::encoding::varint::VarInt;

use super::{deserialize_bytes, serialize_bytes};

macro_rules! assert_serialization {
    ($data:expr => $to:expr) => {{
        use crate::encoding::ser::Serialize;
        assert_eq!($data.size(), $to.len(), "wrong size estimate");

        let res = serialize_bytes($data).unwrap();
        assert_eq!(res.as_ref(), $to);
    }};
}

macro_rules! assert_deserialization {
    ($data:expr => $to:expr) => {{
        #[inline]
        fn eq<T: PartialEq<T>>(a: &T, b: &T) -> bool {
            a == b
        }

        let a = deserialize_bytes($data).unwrap();
        assert!(eq(&a, &$to));
    }};
}

#[test]
fn serialize_string() {
    assert_serialization!("ciao" => b"\x04ciao");
}

// #[test]
// fn serialize_struct() {
//     #[derive(Serialize)]
//     struct MyStruct {
//         a: (u8, u32, u16),
//         b: u32,
//     }

//     let instance = MyStruct {
//         a: (1, 2, 3),
//         b: 10,
//     };

//     assert_serialization!(
//         instance =>
//         &[0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x03, 0x00, 0x00, 0x00, 0x0A]
//     );
// }

#[test]
fn serialize_option() {
    assert_serialization!(Option::<()>::None => &[0x00]);
    assert_serialization!(Some(10u32) => &[0x01, 0x00, 0x00, 0x00, 0x0A]);
}

#[test]
fn serialize_varint() {
    assert_serialization!(VarInt(-1) => &[0xff, 0xff, 0xff, 0xff, 0x0f]);
}

#[test]
fn deserialize() {
    assert_deserialization!(&[0x00, 0x00, 0x00, 0x01] => 1u32);
}

#[test]
fn deserialize_str() {
    assert_deserialization!(b"\x04ciao_extradata" => "ciao");
}

#[test]
fn deserialize_borrowed_bytes() {
    assert_deserialization!(b"\x04aaaa" => &b"aaaa"[..]);
}

#[test]
fn deserialize_owned_bytes() {
    assert_deserialization!(b"\x04aaaa" => Vec::from("aaaa"));
}

// #[test]
// fn deserialize_struct() {
//     #[derive(Deserialize, PartialEq)]
//     struct MyStruct<'a> {
//         borrowed_str: &'a str,
//         borrowed_bytes: &'a [u8],
//         value: u32,
//     }

//     let instance = MyStruct {
//         borrowed_str: "ciao",
//         borrowed_bytes: b"ciao",
//         value: 10,
//     };

//     assert_deserialization!(b"\x04ciao\x04ciao\x00\x00\x00\x0A" => instance);
// }

#[test]
fn deserialize_varint() {
    assert_deserialization!(b"\xff\x01" => VarInt(255));
}

#[test]
fn deserialize_option() {
    assert_deserialization!(&[0x01, 0x01] => Some(1u8));
    assert_deserialization!(&[0x00] => Option::<()>::None);
}
