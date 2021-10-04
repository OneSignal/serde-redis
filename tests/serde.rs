#[macro_use]
extern crate serde_derive;

use redis::Value;
use serde::{Deserialize, Serialize};
use serde_redis::{Deserializer, Serializer};

#[test]
fn test_serde_struct() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct Foo {
        f1: String,
        f2: i32,
        f3: Option<i64>,
        f4: Option<i128>
    }

    let original = Foo {
        f1: "hello world".to_owned(),
        f2: 1,
        f3: Some(2),
        f4: None
    };
    let v = serde_redis::encode::to_vec(&original).unwrap();
    let data = &Value::Bulk(v);
    let de = Deserializer::new(data);
    let decoded: Foo = Deserialize::deserialize(de).unwrap();

    println!("{:?}", original);
    println!("{:?}", decoded);
    assert_eq!(original, decoded);
}
