#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate redis;
extern crate redis_serde;

use std::collections::HashMap;

use serde::Deserialize;
use redis_serde::Deserializer;

use redis::Value;


#[test]
fn deserialize_unit_struct_string() {
    let v = Value::Data(b"hello".to_vec());

    #[derive(Deserialize, Debug, PartialEq)]
    struct Unit(String);

    let mut de = Deserializer::new(v).unwrap();
    let actual: Unit = Deserialize::deserialize(&mut de).unwrap();

    let expected = Unit("hello".to_owned());
    assert_eq!(expected, actual);
}

#[test]
fn deserialize_unit_struct_u8_redis_int() {
    let num = 5u8;
    let v = Value::Int(num as i64);

    #[derive(Deserialize, Debug, PartialEq)]
    struct IntUnit(u8);

    let mut de = Deserializer::new(v).unwrap();
    let actual: IntUnit = Deserialize::deserialize(&mut de).unwrap();

    let expected = IntUnit(num);
    assert_eq!(expected, actual);
}

#[test]
fn deserialize_tuple() {
    let v = vec![Value::Int(5), Value::Data(b"hello".to_vec())];

    let mut de = Deserializer::new(Value::Bulk(v)).unwrap();
    let actual: (u8, String) = Deserialize::deserialize(&mut de).unwrap();

    let expected = (5, "hello".to_owned());
    assert_eq!(expected, actual);
}

#[test]
fn deserialize_struct() {
    let v = vec![
        Value::Data(b"a".to_vec()), Value::Data(b"apple".to_vec()),
        Value::Data(b"b".to_vec()), Value::Data(b"banana".to_vec())
    ];

    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let mut de = Deserializer::new(Value::Bulk(v)).unwrap();
    let actual: Simple = Deserialize::deserialize(&mut de).unwrap();

    let expected = Simple {
        a: "apple".to_owned(),
        b: "banana".to_owned(),
    };

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_hash_map_strings() {
    let v = vec![
        Value::Data(b"a".to_vec()), Value::Data(b"apple".to_vec()),
        Value::Data(b"b".to_vec()), Value::Data(b"banana".to_vec())
    ];

    let mut expected = HashMap::new();
    expected.insert("a".to_string(), "apple".to_string());
    expected.insert("b".to_string(), "banana".to_string());

    let mut de = Deserializer::new(Value::Bulk(v)).unwrap();
    let actual: HashMap<String, String> = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_float() {
    let v = Value::Data(b"3.14159".to_vec());

    let expected = "3.14159".parse::<f32>().unwrap();

    let mut de = Deserializer::new(v).unwrap();
    let actual: f32 = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn deserialize_hash_map_string_u8() {
    let v = vec![
        Value::Data(b"a".to_vec()), Value::Data(b"1".to_vec()),
        Value::Data(b"b".to_vec()), Value::Data(b"2".to_vec())
    ];

    let mut expected = HashMap::new();
    expected.insert("a".to_string(), 1);
    expected.insert("b".to_string(), 2);

    let mut de = Deserializer::new(Value::Bulk(v)).unwrap();
    let actual: HashMap<String, u8> = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_struct_out_of_order() {
    let v = vec![
        Value::Data(b"b".to_vec()), Value::Data(b"banana".to_vec()),
        Value::Data(b"a".to_vec()), Value::Data(b"apple".to_vec()),
    ];

    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let mut de = Deserializer::new(Value::Bulk(v)).unwrap();
    let actual: Simple = Deserialize::deserialize(&mut de).unwrap();

    let expected = Simple {
        a: "apple".to_owned(),
        b: "banana".to_owned(),

    };

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_struct_extra_keys() {
    let v = vec![
        Value::Data(b"c".to_vec()), Value::Data(b"cranberry".to_vec()),
        Value::Data(b"b".to_vec()), Value::Data(b"banana".to_vec()),
        Value::Data(b"a".to_vec()), Value::Data(b"apple".to_vec()),
    ];

    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let mut de = Deserializer::new(Value::Bulk(v)).unwrap();
    let actual: Simple = Deserialize::deserialize(&mut de).unwrap();

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
        Apple
    }

    let mut de = Deserializer::new(v).unwrap();
    let actual: Fruit = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Fruit::Orange, actual);
}

#[test]
fn deserialize_option() {
    let mut de = Deserializer::new(Value::Nil).unwrap();
    let actual: Option<u8> = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(None, actual);
}

#[test]
fn deserialize_complex_struct() {
    let v = vec![
        Value::Data(b"num".to_vec()), Value::Data(b"10".to_vec()),
        Value::Data(b"opt".to_vec()), Value::Data(b"yes".to_vec()),
        Value::Data(b"s".to_vec()), Value::Data(b"yarn".to_vec()),
    ];

    #[derive(Debug, Deserialize, PartialEq)]
    struct Complex {
        num: usize,
        opt: Option<String>,
        not: Option<String>,
        s: String,
    }

    let expected = Complex {
        num: 10,
        opt: Some("yes".to_owned()),
        not: None,
        s: "yarn".to_owned()
    };

    let mut de = Deserializer::new(Value::Bulk(v)).unwrap();
    let actual: Complex = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn deserialize_vec_of_strings() {
    let v = vec![
        Value::Data(b"first".to_vec()),
        Value::Data(b"second".to_vec()),
        Value::Data(b"third".to_vec()),
    ];

    let mut de = Deserializer::new(Value::Bulk(v)).unwrap();
    let actual: Vec<String> = Deserialize::deserialize(&mut de).unwrap();

    let expected = vec!["first".to_string(), "second".to_string(), "third".to_string()];
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

    let mut de = Deserializer::new(Value::Bulk(v)).unwrap();
    let actual: Vec<Rank> = Deserialize::deserialize(&mut de).unwrap();

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
    let values =
        Value::Bulk(vec![
            Value::Bulk(vec![
                Value::Data(b"a".to_vec()), Value::Data(b"apple".to_vec()),
                Value::Data(b"b".to_vec()), Value::Data(b"banana".to_vec())
            ]),
            Value::Bulk(vec![
                Value::Data(b"a".to_vec()), Value::Data(b"art".to_vec()),
                Value::Data(b"b".to_vec()), Value::Data(b"bold".to_vec())
            ])
        ]);


    #[derive(Debug, Deserialize, PartialEq)]
    struct Simple {
        a: String,
        b: String,
    }

    let mut de = Deserializer::new(values).unwrap();
    let actual: Vec<Simple> = Deserialize::deserialize(&mut de).unwrap();

    let expected = vec![Simple {
        a: "apple".to_owned(),
        b: "banana".to_owned(),
    }, Simple {
        a: "art".to_owned(),
        b: "bold".to_owned(),
    }];

    assert_eq!(expected, actual);
}
