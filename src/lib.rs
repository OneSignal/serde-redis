use std::borrow::Borrow;

// `encode` and `decode` are used instead of `ser` and `de` to avoid confusion with the serder
// Serializer and Deserializer traits which occupy a similar namespace.
pub mod decode;
pub mod encode;

pub use crate::decode::Deserializer;
pub use crate::encode::Serializer;

/// Use serde Deserialize to build `T` from a `redis::Value`
pub fn from_redis_value<'de, T, RV>(rv: RV) -> decode::Result<T>
where
    T: serde::de::Deserialize<'de>,
    RV: Borrow<redis::Value>,
{
    let value = rv.borrow();
    serde::de::Deserialize::deserialize(Deserializer::new(value))
}

pub trait RedisDeserialize<'de, T>
where
    T: serde::de::Deserialize<'de>,
{
    fn deserialize(&'de self) -> decode::Result<T>;
}

impl<'de, T> RedisDeserialize<'de, T> for redis::Value
where
    T: serde::de::Deserialize<'de>,
{
    fn deserialize(&'de self) -> decode::Result<T> {
        serde::de::Deserialize::deserialize(Deserializer::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redis::Value;

    #[test]
    fn chain_deserialize_works() {
        let v = Value::Bulk(vec![Value::Int(5), Value::Data(b"hello".to_vec())]);

        let actual: (u8, String) = v.deserialize().unwrap();
        let expected = (5, "hello".into());

        assert_eq!(expected, actual);
    }

    #[test]
    fn from_redis_value_works_with_owned() {
        let v = Value::Bulk(vec![Value::Int(5), Value::Data(b"hello".to_vec())]);

        let actual: (u8, String) = from_redis_value(v).unwrap();
        let expected = (5, "hello".into());

        assert_eq!(expected, actual);
    }

    #[test]
    fn from_redis_value_works_with_borrow() {
        let v = Value::Bulk(vec![Value::Int(5), Value::Data(b"hello".to_vec())]);

        let actual: (u8, String) = from_redis_value(&v).unwrap();
        let expected = (5, "hello".into());

        assert_eq!(expected, actual);
    }
}
