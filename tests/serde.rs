#[macro_use]
extern crate serde_derive;

use serde::{Deserialize, Serialize};
use serde_redis::{Deserializer, Serializer};

#[test]
fn test_serde_struct() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct Foo {
        f1: String,
        f2: i32,
        f3: Option<i64>,
        f4: Option<i128>,
        f5: bool
    }

    let original = Foo {
        f1: "hello world".to_owned(),
        f2: 1,
        f3: Some(2),
        f4: None,
        f5: true
    };
    let value = original.serialize(Serializer).unwrap();
    let de = Deserializer::new(value);
    let decoded: Foo = Deserialize::deserialize(de).unwrap();

    assert_eq!(original, decoded);
}

#[test]
fn serialize_byte_buf() {
    let original = b"0123";
    let value = original.serialize(Serializer).unwrap();
    let de = Deserializer::new(value);
    let decoded: Vec<u8> = Deserialize::deserialize(de).unwrap();

    assert_eq!(original.to_vec(), decoded);
}
