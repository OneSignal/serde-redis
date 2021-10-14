#[macro_use]
extern crate serde_derive;

use redis::Value;
use serde::{Deserialize, Serialize};
use serde_redis::{Deserializer, Serializer};
use std::collections::HashMap;

#[test]
fn serialize_unit_struct_string() {
    #[derive(Serialize, Debug, PartialEq)]
    struct Unit(String);
    let v = Unit("hello".to_owned());

    let actual = v.serialize(Serializer).unwrap();
    let expected = Value::Data(b"hello".to_vec());

    assert_eq!(expected, actual);
}

#[test]
fn serialize_unit_struct_u8_redis_int() {
    #[derive(Serialize, Debug, PartialEq)]
    struct IntUnit(u8);
    let v = IntUnit(5u8);

    let actual = v.serialize(Serializer).unwrap();
    let expected = Value::Data(b"5".to_vec());

    assert_eq!(expected, actual);
}

#[test]
fn serialize_bool() {
    #[derive(Serialize, Debug, PartialEq)]
    struct Bool((bool, bool));
    let v = Bool((true, false));

    let actual = v.serialize(Serializer).unwrap();
    let expected = Value::Bulk(vec![Value::Data(b"1".to_vec()), Value::Data(b"0".to_vec())]);

    assert_eq!(expected, actual);
}

#[test]
fn serialize_tuple() {
    #[derive(Serialize, Debug, PartialEq)]
    struct Tuple((i32, String));
    let v = Tuple((5, "hello".to_owned()));

    let actual = v.serialize(Serializer).unwrap();
    let expected = Value::Bulk(vec![
        Value::Data(b"5".to_vec()),
        Value::Data(b"hello".to_vec()),
    ]);

    assert_eq!(expected, actual);

    // Test decoder
    let de = Deserializer::new(&expected);
    let decoded: (i32, String) = Deserialize::deserialize(de).unwrap();
    assert_eq!(decoded, v.0);
}

#[test]
fn serialize_hash_map_strings() {
    let mut v = HashMap::new();
    v.insert("a".to_string(), "apple".to_string());
    v.insert("b".to_string(), "banana".to_string());

    let actual = v.serialize(Serializer).unwrap();

    // HashMap is not sorted
    if let Value::Bulk(values) = actual {
        assert!(values.contains(&Value::Data(b"a".to_vec())));
        assert!(values.contains(&Value::Data(b"apple".to_vec())));
        assert!(values.contains(&Value::Data(b"b".to_vec())));
        assert!(values.contains(&Value::Data(b"banana".to_vec())));
    }
}

#[test]
fn serialize_float() {
    let v = "3.14159".parse::<f32>().unwrap();

    let actual = v.serialize(Serializer).unwrap();
    let expected = Value::Data(b"3.14159".to_vec());

    assert_eq!(actual, expected);
}

#[test]
fn serialize_hash_map_string_u8() {
    let mut v = HashMap::new();
    v.insert("a".to_string(), 1);
    v.insert("b".to_string(), 2);

    let actual = v.serialize(Serializer).unwrap();
    // HashMap is not sorted
    if let Value::Bulk(values) = actual {
        assert!(values.contains(&Value::Data(b"a".to_vec())));
        assert!(values.contains(&Value::Data(b"1".to_vec())));
        assert!(values.contains(&Value::Data(b"b".to_vec())));
        assert!(values.contains(&Value::Data(b"2".to_vec())));
    }
}

#[test]
fn serialize_enum() {
    #[derive(Debug, Serialize, PartialEq)]
    enum Fruit {
        Orange,
        Apple,
    }

    let v = (Fruit::Orange, Fruit::Apple);

    let actual = v.serialize(Serializer).unwrap();
    let expected = Value::Bulk(vec![
        Value::Data(b"Orange".to_vec()),
        Value::Data(b"Apple".to_vec()),
    ]);

    assert_eq!(expected, actual);
}

#[test]
fn serialize_option() {
    let v: Option<i8> = None;

    let actual = v.serialize(Serializer).unwrap();

    assert_eq!(Value::Nil, actual);
}

#[test]
fn serialize_vec_of_strings() {
    let v = vec![
        "first".to_string(),
        "second".to_string(),
        "third".to_string(),
    ];

    let actual = v.serialize(Serializer).unwrap();
    let expected = Value::Bulk(vec![
        Value::Data(b"first".to_vec()),
        Value::Data(b"second".to_vec()),
        Value::Data(b"third".to_vec()),
    ]);

    assert_eq!(expected, actual);
}

#[test]
fn serialize_vec_of_newtype() {
    #[derive(Debug, PartialEq, Serialize)]
    struct Rank(String);

    let v = vec![
        Rank("first".into()),
        Rank("second".into()),
        Rank("third".into()),
    ];

    let actual = v.serialize(Serializer).unwrap();

    let expected = Value::Bulk(vec![
        Value::Data(b"first".to_vec()),
        Value::Data(b"second".to_vec()),
        Value::Data(b"third".to_vec()),
    ]);

    assert_eq!(expected, actual);
}

#[test]
fn serialize_pipelined_hmap() {
    #[derive(Debug, Serialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let v = vec![
        Simple {
            a: "apple".to_owned(),
            b: "banana".to_owned(),
        },
        Simple {
            a: "art".to_owned(),
            b: "bold".to_owned(),
        },
    ];

    let actual = v.serialize(Serializer).unwrap();
    let expected = Value::Bulk(vec![
        Value::Bulk(vec![
            Value::Data(b"a".to_vec()),
            Value::Data(b"apple".to_vec()),
            Value::Data(b"b".to_vec()),
            Value::Data(b"banana".to_vec()),
        ]),
        Value::Bulk(vec![
            Value::Data(b"a".to_vec()),
            Value::Data(b"art".to_vec()),
            Value::Data(b"b".to_vec()),
            Value::Data(b"bold".to_vec()),
        ]),
    ]);

    assert_eq!(expected, actual);
}

#[test]
fn serialize_pipelined_single_hmap() {
    #[derive(Debug, Serialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let v = vec![Simple {
        a: "apple".to_owned(),
        b: "banana".to_owned(),
    }];

    let actual = v.serialize(Serializer).unwrap();

    let expected = Value::Bulk(vec![Value::Bulk(vec![
        Value::Data(b"a".to_vec()),
        Value::Data(b"apple".to_vec()),
        Value::Data(b"b".to_vec()),
        Value::Data(b"banana".to_vec()),
    ])]);

    assert_eq!(expected, actual);
}

#[test]
fn serialize_struct_with_newtype_field() {
    #[derive(Debug, Serialize, PartialEq)]
    struct Fruit(String);

    #[derive(Debug, Serialize, PartialEq)]
    struct Simple {
        a: Fruit,
        b: Fruit,
    }
    let v = Simple {
        a: Fruit(String::from("apple")),
        b: Fruit(String::from("banana")),
    };

    let actual = v.serialize(Serializer).unwrap();

    let expected = Value::Bulk(vec![
        Value::Data(b"a".to_vec()),
        Value::Data(b"apple".to_vec()),
        Value::Data(b"b".to_vec()),
        Value::Data(b"banana".to_vec()),
    ]);

    assert_eq!(expected, actual);
}

#[test]
fn serialize_byte_buf() {
    let v = b"0123";
    let actual = v.serialize(Serializer).unwrap();

    let expected = Value::Bulk(vec![
        Value::Data(b"48".to_vec()),
        Value::Data(b"49".to_vec()),
        Value::Data(b"50".to_vec()),
        Value::Data(b"51".to_vec()),
    ]);
    assert_eq!(expected, actual);
}

#[test]
fn serialize_pipelined_single_hmap_newtype_fields() {
    #[derive(Debug, Serialize, PartialEq)]
    struct Fruit(String);

    #[derive(Debug, Serialize, PartialEq)]
    struct Simple {
        a: Fruit,
        b: Fruit,
    }

    let v = vec![Simple {
        a: Fruit(String::from("apple")),
        b: Fruit(String::from("banana")),
    }];

    let actual = v.serialize(Serializer).unwrap();

    let expected = Value::Bulk(vec![Value::Bulk(vec![
        Value::Data(b"a".to_vec()),
        Value::Data(b"apple".to_vec()),
        Value::Data(b"b".to_vec()),
        Value::Data(b"banana".to_vec()),
    ])]);

    assert_eq!(expected, actual);
}

#[derive(Debug, Serialize)]
pub struct Details {
    pub time: i64,
    pub count: u32,
    pub ids: Vec<String>,
}

type MapMapList = ::std::collections::HashMap<String, Details>;

#[test]
fn serialize_nested_map_map_list() {
    let mut v = MapMapList::new();
    v.insert(
        "key".to_string(),
        Details {
            time: 1473359995,
            count: 4,
            ids: vec![
                String::from("00000000-0000-0000-0000-000000000000"),
                String::from("00000000-0000-0000-0000-000000000001"),
                String::from("00000000-0000-0000-0000-000000000002"),
            ],
        },
    );

    let actual = v.serialize(Serializer).unwrap();

    let expected = Value::Bulk(vec![
        Value::Data(b"key".to_vec()),
        Value::Bulk(vec![
            Value::Data(b"time".to_vec()),
            Value::Data(b"1473359995".to_vec()),
            Value::Data(b"count".to_vec()),
            Value::Data(b"4".to_vec()),
            Value::Data(b"ids".to_vec()),
            Value::Bulk(vec![
                Value::Data(b"00000000-0000-0000-0000-000000000000".to_vec()),
                Value::Data(b"00000000-0000-0000-0000-000000000001".to_vec()),
                Value::Data(b"00000000-0000-0000-0000-000000000002".to_vec()),
            ]),
        ]),
    ]);

    assert_eq!(expected, actual);
}

#[test]
fn serialize_nested_item() {
    let v = vec![vec![vec!["hi".to_string()]]];

    let actual = v.serialize(Serializer).unwrap();

    let expected = Value::Bulk(vec![Value::Bulk(vec![Value::Bulk(vec![Value::Data(
        b"hi".to_vec(),
    )])])]);
    assert_eq!(expected, actual);
}
