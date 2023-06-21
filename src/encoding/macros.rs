#[macro_export]
/// Asserts that the input type `$data: Serialize` serializes into
/// `$to: &[u8]`
macro_rules! assert_serialization {
    ($data:expr => $to:expr) => {{
        use $crate::encoding::{ser::Serialize, serialize_bytes};
        assert_eq!($data.size(), $to.len(), "wrong size estimate");

        let res = serialize_bytes($data);
        assert_eq!(res.as_ref(), $to);
    }};
}

#[macro_export]
/// Asserts that the input data `$data: &[u8]` deserializes
/// into `$to: Deserialize`
macro_rules! assert_deserialization {
    ($data:expr => $to:expr, $type:ty) => {{
        use core::ops::Deref;
        use $crate::encoding::deserialize_bytes;

        let a: $type = deserialize_bytes(&$data[..]).unwrap();
        assert!($to.eq(a.deref()))
    }};

    ($data:expr => $to:expr) => {{
        #[inline]
        fn eq<T: PartialEq<T>>(a: &T, b: &T) -> bool {
            a == b
        }

        use $crate::encoding::deserialize_bytes;

        let a = deserialize_bytes(&$data[..]).unwrap();
        assert!(eq(&a, &$to))
    }};
}
