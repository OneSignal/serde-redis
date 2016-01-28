redis-serde
===========

[serde][] serialization and deserialization of [redis-rs][] values

[serde]: https://github.com/serde-rs/serde
[redis-rs]: https://github.com/mitsuhiko/redis-rs

**NOT READY FOR USE**

## Summary

This crate gives you automatic deserialization of values returned from redis-rs.

```rust
#[derive(Debug, Deserialize, PartialEq)]
struct Simple {
    a: String,
    b: String,
}

let s : Simple = try!(redis_connection.hgetall("simple_hash"));
```

## Future work

- Work at the redis protocol level instead of `redis::Value` type.
- Merge into redis-rs?
