extern crate serde;
extern crate redis;

// `encode` and `decode` are used instead of `ser` and `de` to avoid confusion with the serder
// Serializer and Deserializer traits which occupy a similar namespace.
pub mod encode;
pub mod decode;

pub use encode::Serializer;
pub use decode::Deserializer;

/// Use serde Deserialize to build `T` from a `redis::Value`
pub fn from_redis_value<'de, T>(rv: ::redis::Value) -> decode::Result<T>
    where T: serde::de::Deserialize<'de>
{
    ::serde::de::Deserialize::deserialize(Deserializer::new(rv))
}

pub trait RedisDeserialize<'de, T>
    where T: serde::de::Deserialize<'de>
{
    fn deserialize(self) -> decode::Result<T>;
}

impl<'de, T> RedisDeserialize<'de, T> for redis::Value
    where T: serde::de::Deserialize<'de>
{
    fn deserialize(self) -> decode::Result<T> {
        ::serde::de::Deserialize::deserialize(Deserializer::new(self))
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
    fn from_redis_value_works() {
        let v = Value::Bulk(vec![Value::Int(5), Value::Data(b"hello".to_vec())]);

        let actual: (u8, String) = from_redis_value(v).unwrap();
        let expected = (5, "hello".into());

        assert_eq!(expected, actual);
    }
}
