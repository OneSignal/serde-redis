#[macro_use]
extern crate serde_derive;

extern crate redis;
extern crate serde;
extern crate serde_bytes;
extern crate serde_redis;

use std::collections::HashMap;

use serde::Deserialize;
use serde_redis::Deserializer;

use redis::Value;

#[test]
fn deserialize_unit_struct_string() {
    let v = Value::Data(b"hello".to_vec());

    #[derive(Deserialize, Debug, PartialEq)]
    struct Unit(String);

    let de = Deserializer::new(&v);
    let actual: Unit = Deserialize::deserialize(de).unwrap();

    let expected = Unit("hello".to_owned());
    assert_eq!(expected, actual);
}

#[test]
fn deserialize_unit_struct_u8_redis_int() {
    let num = 5u8;
    let v = Value::Int(num as i64);

    #[derive(Deserialize, Debug, PartialEq)]
    struct IntUnit(u8);

    let de = Deserializer::new(&v);
    let actual: IntUnit = Deserialize::deserialize(de).unwrap();

    let expected = IntUnit(num);
    assert_eq!(expected, actual);
}

#[test]
fn deserialize_bool() {
    let v = vec![
        Value::Data(b"0".to_vec()),
        Value::Data(b"false".to_vec()),
        Value::Data(b"False".to_vec()),
        Value::Data(b"1".to_vec()),
        Value::Data(b"true".to_vec()),
        Value::Data(b"True".to_vec()),
    ];

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: Vec<bool> = Deserialize::deserialize(de).unwrap();

    let expected = [false, false, false, true, true, true];
    assert_eq!(&expected, &actual[..]);
}

#[test]
fn deserialize_tuple() {
    let v = vec![Value::Int(5), Value::Data(b"hello".to_vec())];

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: (u8, String) = Deserialize::deserialize(de).unwrap();

    let expected = (5, "hello".to_owned());
    assert_eq!(expected, actual);
}

#[test]
fn deserialize_struct() {
    let v = vec![
        Value::Data(b"a".to_vec()),
        Value::Data(b"apple".to_vec()),
        Value::Data(b"b".to_vec()),
        Value::Data(b"banana".to_vec()),
    ];

    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: Simple = Deserialize::deserialize(de).unwrap();

    let expected = Simple {
        a: "apple".to_owned(),
        b: "banana".to_owned(),
    };

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_hash_map_strings() {
    let v = vec![
        Value::Data(b"a".to_vec()),
        Value::Data(b"apple".to_vec()),
        Value::Data(b"b".to_vec()),
        Value::Data(b"banana".to_vec()),
    ];

    let mut expected = HashMap::new();
    expected.insert("a".to_string(), "apple".to_string());
    expected.insert("b".to_string(), "banana".to_string());

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: HashMap<String, String> = Deserialize::deserialize(de).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_float() {
    let v = Value::Data(b"3.14159".to_vec());

    let expected = "3.14159".parse::<f32>().unwrap();

    let de = Deserializer::new(&v);
    let actual: f32 = Deserialize::deserialize(de).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_hash_map_string_u8() {
    let v = vec![
        Value::Data(b"a".to_vec()),
        Value::Data(b"1".to_vec()),
        Value::Data(b"b".to_vec()),
        Value::Data(b"2".to_vec()),
    ];

    let mut expected = HashMap::new();
    expected.insert("a".to_string(), 1);
    expected.insert("b".to_string(), 2);

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: HashMap<String, u8> = Deserialize::deserialize(de).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_struct_out_of_order() {
    let v = vec![
        Value::Data(b"b".to_vec()),
        Value::Data(b"banana".to_vec()),
        Value::Data(b"a".to_vec()),
        Value::Data(b"apple".to_vec()),
    ];

    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: Simple = Deserialize::deserialize(de).unwrap();

    let expected = Simple {
        a: "apple".to_owned(),
        b: "banana".to_owned(),
    };

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_struct_extra_keys() {
    let v = vec![
        Value::Data(b"c".to_vec()),
        Value::Data(b"cranberry".to_vec()),
        Value::Data(b"b".to_vec()),
        Value::Data(b"banana".to_vec()),
        Value::Data(b"a".to_vec()),
        Value::Data(b"apple".to_vec()),
    ];

    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: Simple = Deserialize::deserialize(de).unwrap();

    let expected = Simple {
        a: "apple".to_owned(),
        b: "banana".to_owned(),
    };

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_enum() {
    let v = Value::Data(b"Orange".to_vec());

    #[derive(Debug, Deserialize, PartialEq)]
    enum Fruit {
        Orange,
        Apple,
    }

    let de = Deserializer::new(&v);
    let actual: Fruit = Deserialize::deserialize(de).unwrap();

    assert_eq!(Fruit::Orange, actual);
}

#[test]
fn deserialize_option() {
    let de = Deserializer::new(&Value::Nil);
    let actual: Option<u8> = Deserialize::deserialize(de).unwrap();

    assert_eq!(None, actual);
}

#[test]
fn deserialize_complex_struct() {
    let v = vec![
        Value::Data(b"num".to_vec()),
        Value::Data(b"10".to_vec()),
        Value::Data(b"opt".to_vec()),
        Value::Data(b"yes".to_vec()),
        Value::Data(b"s".to_vec()),
        Value::Data(b"yarn".to_vec()),
    ];

    #[derive(Debug, Deserialize, PartialEq)]
    struct Complex {
        num: usize,
        opt: Option<String>,
        not_present: Option<String>,
        s: String,
    }

    let expected = Complex {
        num: 10,
        opt: Some("yes".to_owned()),
        not_present: None,
        s: "yarn".to_owned(),
    };

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: Complex = Deserialize::deserialize(de).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_vec_of_strings() {
    let v = vec![
        Value::Data(b"first".to_vec()),
        Value::Data(b"second".to_vec()),
        Value::Data(b"third".to_vec()),
    ];

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: Vec<String> = Deserialize::deserialize(de).unwrap();

    let expected = vec![
        "first".to_string(),
        "second".to_string(),
        "third".to_string(),
    ];
    assert_eq!(expected, actual);
}

#[test]
fn deserialize_vec_of_newtype() {
    let v = vec![
        Value::Data(b"first".to_vec()),
        Value::Data(b"second".to_vec()),
        Value::Data(b"third".to_vec()),
    ];

    #[derive(Debug, PartialEq, Deserialize)]
    struct Rank(String);

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: Vec<Rank> = Deserialize::deserialize(de).unwrap();

    let expected = vec![
        Rank("first".into()),
        Rank("second".into()),
        Rank("third".into()),
    ];

    assert_eq!(expected, actual);
}

/// Test for deserializing pipelined structs
///
/// In pipeline mode, there is nested bulk items that are returned. The original implementation did
/// not handle this.
#[test]
fn deserialize_pipelined_hmap() {
    let values = Value::Bulk(vec![
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

    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let de = Deserializer::new(&values);
    let actual: Vec<Simple> = Deserialize::deserialize(de).unwrap();

    let expected = vec![
        Simple {
            a: "apple".to_owned(),
            b: "banana".to_owned(),
        },
        Simple {
            a: "art".to_owned(),
            b: "bold".to_owned(),
        },
    ];

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_pipelined_single_hmap() {
    let values = Value::Bulk(vec![Value::Bulk(vec![
        Value::Data(b"a".to_vec()),
        Value::Data(b"apple".to_vec()),
        Value::Data(b"b".to_vec()),
        Value::Data(b"banana".to_vec()),
    ])]);

    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let de = Deserializer::new(&values);
    let actual: Vec<Simple> = Deserialize::deserialize(de).unwrap();

    let expected = vec![Simple {
        a: "apple".to_owned(),
        b: "banana".to_owned(),
    }];

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_struct_with_newtype_field() {
    let v = vec![
        Value::Data(b"b".to_vec()),
        Value::Data(b"banana".to_vec()),
        Value::Data(b"a".to_vec()),
        Value::Data(b"apple".to_vec()),
    ];

    #[derive(Debug, Deserialize, PartialEq)]
    struct Fruit(String);

    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: Fruit,
        b: Fruit,
    }

    let data = Value::Bulk(v);

    let de = Deserializer::new(&data);
    let actual: Simple = Deserialize::deserialize(de).unwrap();

    let expected = Simple {
        a: Fruit(String::from("apple")),
        b: Fruit(String::from("banana")),
    };

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_byte_buf() {
    let data = Value::Data(b"0000".to_vec());
    let de = Deserializer::new(&data);
    let actual: serde_bytes::ByteBuf = Deserialize::deserialize(de).unwrap();

    let expected = b"0000";
    assert_eq!(expected, &actual[..]);
}

#[test]
fn deserialize_pipelined_single_hmap_newtype_fields() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Fruit(String);

    let values = Value::Bulk(vec![Value::Bulk(vec![
        Value::Data(b"a".to_vec()),
        Value::Data(b"apple".to_vec()),
        Value::Data(b"b".to_vec()),
        Value::Data(b"banana".to_vec()),
    ])]);

    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: Fruit,
        b: Fruit,
    }

    let de = Deserializer::new(&values);
    let actual: Vec<Simple> = Deserialize::deserialize(de).unwrap();

    let expected = vec![Simple {
        a: Fruit(String::from("apple")),
        b: Fruit(String::from("banana")),
    }];

    assert_eq!(expected, actual);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Details {
    pub time: i64,
    pub count: u32,
    pub ids: Vec<String>,
}

type MapMapList = ::std::collections::HashMap<String, Details>;

#[test]
fn deserialize_nested_map_map_list() {
    let value = Value::Bulk(vec![
        Value::Data(b"key".to_vec()),
        Value::Bulk(vec![
            Value::Data(b"count".to_vec()),
            Value::Data(b"4".to_vec()),
            Value::Data(b"time".to_vec()),
            Value::Data(b"1473359995".to_vec()),
            Value::Data(b"ids".to_vec()),
            Value::Bulk(vec![
                Value::Data(b"00000000-0000-0000-0000-000000000000".to_vec()),
                Value::Data(b"00000000-0000-0000-0000-000000000001".to_vec()),
                Value::Data(b"00000000-0000-0000-0000-000000000002".to_vec()),
            ]),
        ]),
    ]);

    let de = Deserializer::new(&value);
    let map: MapMapList = Deserialize::deserialize(de).unwrap();

    let nest = map.get("key").unwrap();
    assert_eq!(nest.count, 4);
    assert_eq!(nest.time, 1473359995);
    assert_eq!(
        &nest.ids[..],
        &[
            String::from("00000000-0000-0000-0000-000000000000"),
            String::from("00000000-0000-0000-0000-000000000001"),
            String::from("00000000-0000-0000-0000-000000000002")
        ]
    );
}

#[test]
#[should_panic]
fn deserialize_nested_item() {
    let value = Value::Bulk(vec![Value::Bulk(vec![Value::Data(b"hi".to_vec())])]);

    let de = Deserializer::new(&value);
    let _hellos: Vec<String> = Deserialize::deserialize(de).unwrap();
}
