mod ser {
    use crate::{assert_serialization, encoding::varint::VarInt};

    #[test]
    fn serialize_string() {
        assert_serialization!("ciao" => b"\x04ciao");
    }

    #[test]
    fn serialize_option() {
        assert_serialization!(Option::<()>::None => &[0x00]);
        assert_serialization!(Some(10u32) => &[0x01, 0x00, 0x00, 0x00, 0x0A]);
    }

    #[test]
    fn serialize_varint() {
        assert_serialization!(VarInt(-1) => &[0xff, 0xff, 0xff, 0xff, 0x0f]);
    }
}

mod de {
    use crate::{assert_deserialization, encoding::varint::VarInt};

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

    #[test]
    fn deserialize_varint() {
        assert_deserialization!(b"\xff\x01" => VarInt(255));
    }

    #[test]
    fn deserialize_option() {
        assert_deserialization!(&[0x01, 0x01] => Some(1u8));
        assert_deserialization!(&[0x00] => Option::<()>::None);
    }
}
