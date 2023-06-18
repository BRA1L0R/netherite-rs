mod serialize {
    use netherite::{assert_serialization, Serialize};
    use std::mem::size_of;

    #[test]
    fn derive_empty() {
        #[derive(Serialize)]
        struct MyStruct {}
    }

    #[test]
    fn derive_fields() {
        #[derive(Serialize, Default)]
        struct MyStruct {
            field1: u32,
            field2: u16,
            unit: (),
            field3: u16,
        }

        let instance = MyStruct::default();

        assert_eq!(instance.size(), size_of::<u64>());
        assert_serialization!(instance => &[0; size_of::<u64>()]);
    }

    #[test]
    fn derive_generic() {
        #[derive(Serialize)]
        struct MyStruct<T> {
            unit: T,
        }

        let instance = MyStruct { unit: 0u32 };
        assert_eq!(instance.size(), std::mem::size_of::<u32>());
        assert_serialization!(instance => &[0; size_of::<u32>()]);
    }

    #[test]
    fn derive_lifetime() {
        #[derive(Serialize)]
        struct MyStruct<'a> {
            unit: &'a u32,
        }

        let instance = MyStruct { unit: &0u32 };
        assert_eq!(instance.size(), std::mem::size_of::<u32>());
        assert_serialization!(instance => &[0; size_of::<u32>()]);
    }
}

#[allow(dead_code)]
mod deserialize {
    use std::mem::size_of;

    use netherite::{assert_deserialization, Deserialize};

    #[test]
    fn derive_empty() {
        #[derive(Deserialize)]
        struct MyStruct {}
    }

    #[test]
    fn derive_fields() {
        #[derive(Deserialize, Default, PartialEq)]
        struct MyStruct {
            field1: u32,
            field2: u32,
            unit: (),
        }

        let instance = MyStruct::default();
        assert_deserialization!(&[0; size_of::<u64>()] => instance);
    }

    #[test]
    fn derive_generic() {
        #[derive(Deserialize, Default, PartialEq)]
        struct MyStruct<T> {
            generic: T,
        }

        let instance: MyStruct<u32> = MyStruct::default();
        assert_deserialization!(&[0; size_of::<u32>()] => instance);
    }

    #[test]
    fn derive_lifetime() {
        #[derive(Deserialize, Default, PartialEq)]
        struct MyStruct<'a, T> {
            field1: &'a [u8],
            field2: T,
        }

        let instance: MyStruct<'_, &[u8]> = MyStruct::default();
        assert_deserialization!(&[0, 0] => instance);
    }
}
