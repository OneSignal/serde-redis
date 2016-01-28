redis-serde
===========

[serde][] serialization and deserialization of [redis-rs][] values

[serde]: https://github.com/serde-rs/serde
[redis-rs]: https://github.com/mitsuhiko/redis-rs

## Status

- Deserialization: Works
- Serialization: Completely **unimplemented**

## Summary

This crate gives you automatic deserialization of values returned from redis-rs.

```rust
use redis_serde::from_redis_value;

#[derive(Debug, Deserialize, PartialEq)]
struct Simple {
    a: String,
    b: String,
}

let s: Simple = from_redis_value(try!(redis_connection.hgetall("simple_hash")));
```

## Future work

- Work at the redis protocol level instead of `redis::Value` type.
- Merge into redis-rs?
