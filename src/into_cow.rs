use redis::Value;
use std::borrow::Cow;

/// A value that can be turned into a `Cow<'a, Value>`. This is primarily useful
/// because there is not an `impl Into<Cow<'a, Value>>` built-in for
/// `redis::Value` and `&'a redis::Value`. This allows `from_redis_value` to take
/// either one of those.
pub trait IntoCow<'a> {
    fn into_cow(self) -> Cow<'a, Value>;
}

impl<'a> IntoCow<'a> for &'a Value {
    fn into_cow(self) -> Cow<'a, Value> {
        Cow::Borrowed(self)
    }
}

impl IntoCow<'static> for Value {
    fn into_cow(self) -> Cow<'static, Value> {
        Cow::Owned(self)
    }
}
