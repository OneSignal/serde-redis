redis-serde
===========

[![Build Status](https://travis-ci.org/OneSignal/serde-redis.svg?branch=master)](https://travis-ci.org/OneSignal/serde-redis)
[![Documentation](https://docs.rs/serde-redis/badge.svg)](https://docs.rs/crate/serde-redis/)
[![Crates.io Version](https://img.shields.io/crates/v/serde-redis.svg)](https://crates.io/crates/serde-redis/)


[serde][] serialization and deserialization of [redis-rs][] values

[serde]: https://github.com/serde-rs/serde
[redis-rs]: https://github.com/mitsuhiko/redis-rs

## Status

- Deserialization: Everything _should_ work.
- Serialization: **unimplemented**

## Summary

This crate gives you automatic deserialization of values returned from redis-rs.

```rust
use serde_redis::RedisDeserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Simple {
    a: String,
    b: String,
}

let s: Simple = redis.hgetall("simple_hash")?
                     .deserialize()?;
```

## Future work

- Work at the redis protocol level instead of `redis::Value` type.
- Merge into redis-rs?

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

