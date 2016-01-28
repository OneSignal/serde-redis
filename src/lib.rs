extern crate serde;
extern crate redis;

// `encode` and `decode` are used instead of `ser` and `de` to avoid confusion with the serder
// Serializer and Deserializer traits which occupy a similar namespace.
pub mod encode;
pub mod decode;

pub use encode::Serializer;
pub use decode::Deserializer;
