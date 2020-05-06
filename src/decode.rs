use redis::Value;
use serde::{self, de};
use std::borrow::Cow;
use std::fmt::{self, Display};
use std::iter::Peekable;
use std::{error, num, str, string, vec};

use crate::cow_iter::CowIter;

/// Error that can be produced during deserialization
#[derive(Debug)]
pub enum Error {
    Custom(String),
    EndOfStream,
    UnknownVariant(String, &'static [&'static str]),
    UnknownField(String, &'static [&'static str]),
    MissingField(&'static str),
    DuplicateField(&'static str),
    DeserializeNotSupported,
    WrongValue(String),
    StrFromUtf8(str::Utf8Error),
    StringFromUtf8(string::FromUtf8Error),
    ParseInt(num::ParseIntError),
    ParseFloat(num::ParseFloatError),
}

impl Error {
    pub fn wrong_value<S>(msg: S) -> Error
    where
        S: Into<String>,
    {
        Error::WrongValue(msg.into())
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::StrFromUtf8(ref err) => Some(err),
            Error::StringFromUtf8(ref err) => Some(err),
            Error::ParseInt(ref err) => Some(err),
            Error::ParseFloat(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Custom(ref reason) => write!(f, "CustomError({})", reason),
            Error::EndOfStream => write!(f, "Reached end of stream"),
            Error::UnknownVariant(ref variant, ref expected) => write!(
                f,
                "unexpected variant \"{}\"; expected {:?}",
                variant, expected
            ),
            Error::UnknownField(ref field, ref expected) => {
                write!(f, "unexpected field \"{}\"; expected {:?}", field, expected)
            }
            Error::MissingField(ref field) => write!(f, "missing field {:?}", field),
            Error::DuplicateField(ref field) => write!(f, "duplicate field {:?}", field),
            Error::DeserializeNotSupported => write!(f, "Deserialization option not supported"),
            Error::WrongValue(ref value_type) => write!(f, "Got unexpected value: {}", value_type),
            Error::StrFromUtf8(ref e) => write!(f, "{}", e),
            Error::StringFromUtf8(ref e) => write!(f, "{}", e),
            Error::ParseInt(ref e) => write!(f, "{}", e),
            Error::ParseFloat(ref e) => write!(f, "{}", e),
        }
    }
}

impl de::Error for Error {
    /// Raised when there is general error when deserializing a type.
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }

    /// Raised when a `Deserialize` enum type received an unexpected variant.
    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        Error::UnknownVariant(variant.to_owned(), expected)
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Error {
        Error::UnknownField(field.to_owned(), expected)
    }

    fn missing_field(field: &'static str) -> Error {
        Error::MissingField(field)
    }

    fn duplicate_field(field: &'static str) -> Error {
        Error::DuplicateField(field)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error::StrFromUtf8(err)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error::StringFromUtf8(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::ParseInt(err)
    }
}

impl From<num::ParseFloatError> for Error {
    fn from(err: num::ParseFloatError) -> Error {
        Error::ParseFloat(err)
    }
}

/// deserializes Redis `Value`s
///
/// Deserializes a sequence of redis values. In the case of a Bulk value (eg, a
/// nested list), another deserializer is created for that sequence. The limit
/// to nested sequences is proportional to the maximum stack depth in current
/// machine.
///
/// If creating a Deserializer manually (ie not using `from_redis_value()`), the redis values must
/// first be placed in a Vec.
#[derive(Debug)]
pub struct Deserializer<'a> {
    values: Peekable<vec::IntoIter<Cow<'a, Value>>>,
}

pub trait AsValueVec<'a> {
    fn as_value_vec(self) -> Vec<Cow<'a, Value>>;
}

impl<'a> AsValueVec<'a> for &'a Value {
    #[inline]
    fn as_value_vec(self) -> Vec<Cow<'a, Value>> {
        vec![Cow::Borrowed(self)]
    }
}

impl<'a> AsValueVec<'a> for Cow<'a, Value> {
    #[inline]
    fn as_value_vec(self) -> Vec<Cow<'a, Value>> {
        vec![self]
    }
}

impl AsValueVec<'static> for Value {
    #[inline]
    fn as_value_vec(self) -> Vec<Cow<'static, Value>> {
        vec![Cow::Owned(self)]
    }
}

impl<'a> AsValueVec<'a> for Vec<Cow<'a, Value>> {
    #[inline]
    fn as_value_vec(self) -> Vec<Cow<'a, Value>> {
        self
    }
}

impl<'a> Deserializer<'a> {
    pub fn new<V>(values: V) -> Self
    where
        V: AsValueVec<'a>,
    {
        Deserializer {
            values: values.as_value_vec().into_iter().peekable(),
        }
    }

    /// Returns a reference to the next value
    #[inline]
    pub fn peek(&mut self) -> Option<&Value> {
        let val = self.values.peek()?;

        Some(val)
    }

    /// Return the next value
    #[inline]
    pub fn next(&mut self) -> Result<Cow<'a, Value>> {
        match self.values.next() {
            Some(value) => Ok(value),
            None => Err(Error::EndOfStream),
        }
    }

    pub fn next_bulk(&mut self) -> Result<Cow<'a, Vec<Value>>> {
        match self.next()? {
            Cow::Owned(Value::Bulk(values)) => Ok(Cow::Owned(values)),
            Cow::Borrowed(Value::Bulk(values)) => Ok(Cow::Borrowed(values)),
            v @ _ => Err(Error::wrong_value(format!("expected bulk but got {:?}", v))),
        }
    }

    pub fn next_bytes(&mut self) -> Result<Cow<'a, Vec<u8>>> {
        match self.next()? {
            Cow::Owned(Value::Data(bytes)) => Ok(Cow::Owned(bytes)),
            Cow::Borrowed(Value::Data(bytes)) => Ok(Cow::Borrowed(bytes)),
            v => {
                let msg = format!("Expected bytes, but got {:?}", v);
                return Err(Error::wrong_value(msg));
            }
        }
    }

    pub fn read_string(&mut self) -> Result<Cow<'a, str>> {
        let redis_value = self.next()?;
        Ok(match redis_value {
            Cow::Owned(Value::Data(bytes)) => Cow::Owned(String::from_utf8(bytes)?),
            Cow::Borrowed(Value::Data(bytes)) => Cow::Borrowed(str::from_utf8(bytes)?),
            _ => {
                let msg = format!("Expected Data, got {:?}", &redis_value);
                return Err(Error::wrong_value(msg));
            }
        })
    }
}

macro_rules! impl_num {
    ($ty:ty, $deserialize_method:ident, $visitor_method:ident) => {
        #[inline]
        fn $deserialize_method<V>(mut self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            let redis_value = self.next()?;
            let value = match redis_value {
                Cow::Borrowed(Value::Data(bytes)) => {
                    let s = str::from_utf8(bytes)?;
                    s.parse::<$ty>()?
                }
                Cow::Owned(Value::Data(bytes)) => {
                    let s = String::from_utf8(bytes)?;
                    s.parse::<$ty>()?
                }
                Cow::Borrowed(Value::Int(i)) => *i as $ty,
                Cow::Owned(Value::Int(i)) => i as $ty,
                _ => {
                    let msg = format!("Expected Data or Int, got {:?}", &redis_value);
                    return Err(Error::wrong_value(msg));
                }
            };

            visitor.$visitor_method(value)
        }
    };
}

macro_rules! default_deserialize {
    ($($name:ident)*) => {
        $(
            #[inline]
            fn $name<V>(self, visitor: V) -> Result<V::Value>
                where V: de::Visitor<'de>
            {
                self.deserialize_str(visitor)
            }
        )*
    }
}

impl<'a, 'de> serde::Deserializer<'de> for Deserializer<'a> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let buf = self.next_bytes()?;
        match buf {
            Cow::Borrowed(buf) => visitor.visit_bytes(buf),
            Cow::Owned(buf) => visitor.visit_byte_buf(buf),
        }
    }

    #[inline]
    fn deserialize_string<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s = self.read_string()?;
        match s {
            Cow::Borrowed(s) => visitor.visit_str(s),
            Cow::Owned(s) => visitor.visit_string(s),
        }
    }

    #[inline]
    fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s = self.read_string()?;
        match s {
            Cow::Borrowed(s) => visitor.visit_str(s),
            Cow::Owned(s) => visitor.visit_string(s),
        }
    }

    impl_num!(u8, deserialize_u8, visit_u8);
    impl_num!(u16, deserialize_u16, visit_u16);
    impl_num!(u32, deserialize_u32, visit_u32);
    impl_num!(u64, deserialize_u64, visit_u64);

    impl_num!(i8, deserialize_i8, visit_i8);
    impl_num!(i16, deserialize_i16, visit_i16);
    impl_num!(i32, deserialize_i32, visit_i32);
    impl_num!(i64, deserialize_i64, visit_i64);

    impl_num!(f32, deserialize_f32, visit_f32);
    impl_num!(f64, deserialize_f64, visit_f64);

    default_deserialize!(
        deserialize_char
        deserialize_unit
    );

    #[inline]
    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let s = self.read_string()?;

        let b = match s.as_ref() {
            "1" | "true" | "True" => true,
            "0" | "false" | "False" => false,
            _ => {
                return Err(Error::WrongValue(format!(
                    "Expected 1/0/true/false/True/False, got {}",
                    s
                )))
            }
        };

        visitor.visit_bool(b)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let bytes = self.next_bytes()?;
        match bytes {
            Cow::Borrowed(bytes) => visitor.visit_bytes(bytes),
            Cow::Owned(bytes) => visitor.visit_byte_buf(bytes),
        }
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let values = self.next_bulk()?;
        visitor.visit_seq(SeqVisitor {
            iter: CowIter::new(values),
        })
    }

    #[inline]
    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let values = self.next_bulk()?;
        visitor.visit_map(MapVisitor {
            iter: CowIter::new(values),
        })
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        mut self,
        _enum: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(EnumVisitor {
            variant: self.next()?,
            content: Cow::Owned(Value::Nil),
        })
    }

    #[inline]
    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let maybe = match self.peek() {
            Some(v) => match *v {
                Value::Data(_) => Some(()),
                Value::Int(_) => Some(()),
                Value::Nil => None,
                _ => {
                    let msg = format!("Expected Data, Int, or Nil");
                    return Err(Error::wrong_value(msg));
                }
            },
            None => None,
        };

        if maybe.is_some() {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }
}

struct SeqVisitor<'a> {
    iter: CowIter<'a>,
}

impl<'a, 'de> de::SeqAccess<'de> for SeqVisitor<'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(v) => seed.deserialize(Deserializer::new(v)).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

struct MapVisitor<'a> {
    iter: CowIter<'a>,
}

impl<'a, 'de> serde::de::MapAccess<'de> for MapVisitor<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(v) => seed.deserialize(Deserializer::new(v)).map(Some),
            None => Ok(None),
        }
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(v) => seed.deserialize(Deserializer::new(v)),
            None => Err(Error::EndOfStream),
        }
    }
}

struct VariantVisitor<'a> {
    value: Cow<'a, Value>,
}

impl<'a, 'de> serde::de::VariantAccess<'de> for VariantVisitor<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(Deserializer::new(self.value))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        use serde::Deserializer;
        let deserializer = self::Deserializer::new(self.value);
        deserializer.deserialize_any(visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        use serde::Deserializer;
        let deserializer = self::Deserializer::new(self.value);
        deserializer.deserialize_any(visitor)
    }
}

struct EnumVisitor<'a> {
    variant: Cow<'a, Value>,
    content: Cow<'a, Value>,
}

impl<'a, 'de> de::EnumAccess<'de> for EnumVisitor<'a> {
    type Error = Error;
    type Variant = VariantVisitor<'a>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        Ok((
            seed.deserialize(Deserializer::new(self.variant))?,
            VariantVisitor {
                value: self.content,
            },
        ))
    }
}
