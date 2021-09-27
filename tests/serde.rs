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
        f2: String,
        // f2: i32,
        // f3: i64,
    }

    let original = Foo {
        f1: "hello world".to_owned(),
        f2: "hi".to_owned(),
    };
    let v = serde_redis::encode::to_vec(&original).unwrap();
    let data = &Value::Bulk(v);
    let de = Deserializer::new(data);
    let decoded: Foo = Deserialize::deserialize(de).unwrap();

    println!("{:?}", original);
    println!("{:?}", decoded);
    assert_eq!(original, decoded);
}
