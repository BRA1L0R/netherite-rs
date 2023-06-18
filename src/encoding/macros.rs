#[macro_export]
/// Asserts that the input type `$data: Serialize` serializes into
/// `$to: &[u8]`
macro_rules! assert_serialization {
    ($data:expr => $to:expr) => {{
        use $crate::encoding::{ser::Serialize, serialize_bytes};
        assert_eq!($data.size(), $to.len(), "wrong size estimate");

        let res = serialize_bytes($data).unwrap();
        assert_eq!(res.as_ref(), $to);
    }};
}

#[macro_export]
/// Asserts that the input data `$data: &[u8]` deserializes
/// into `$to: Deserialize`
macro_rules! assert_deserialization {
    ($data:expr => $to:expr) => {{
        use $crate::encoding::deserialize_bytes;

        #[inline]
        fn eq<T: PartialEq<T>>(a: &T, b: &T) -> bool {
            a == b
        }

        let a = deserialize_bytes($data).unwrap();
        assert!(eq(&a, &$to));
    }};
}
