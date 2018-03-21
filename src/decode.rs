use std::error;
use std::fmt::{self, Display};
use std::iter::Peekable;
use std::num;
use std::string;
use std::vec;

use redis::Value;

use serde::{self, de};

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
    FromUtf8(string::FromUtf8Error),
    ParseInt(num::ParseIntError),
    ParseFloat(num::ParseFloatError),
}

impl Error {
    pub fn wrong_value<S>(msg: S) -> Error
        where S: Into<String>
    {
        Error::WrongValue(msg.into())
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Custom(_) => "custom error when decoding redis values",
            Error::EndOfStream => "end of redis value stream",
            Error::UnknownField(..) => "unknown field",
            Error::UnknownVariant(..) => "unknown variant",
            Error::MissingField(_) => "missing field",
            Error::DuplicateField(_) => "duplicate field",
            Error::DeserializeNotSupported => "unsupported deserialization operation",
            Error::WrongValue(_) => "expected value of different type",
            Error::FromUtf8(ref err) => err.description(),
            Error::ParseInt(ref err) => err.description(),
            Error::ParseFloat(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::FromUtf8(ref err) => Some(err),
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
            Error::UnknownVariant(ref variant, ref expected) => {
                write!(f, "unexpected variant \"{}\"; expected {:?}", variant, expected)
            }
            Error::UnknownField(ref field, ref expected) => {
                write!(f, "unexpected field \"{}\"; expected {:?}", field, expected)
            }
            Error::MissingField(ref field) => write!(f, "missing field {:?}", field),
            Error::DuplicateField(ref field) => write!(f, "duplicate field {:?}", field),
            Error::DeserializeNotSupported => write!(f, "Deserialization option not supported"),
            Error::WrongValue(ref value_type) => write!(f, "Got unexpected value: {}", value_type),
            Error::FromUtf8(ref e) => write!(f, "{}", e),
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

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error::FromUtf8(err)
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
pub struct Deserializer {
    values: Peekable<vec::IntoIter<Value>>,
}

pub trait IntoValueVec {
    fn into_value_vec(self) -> Vec<Value>;
}

impl IntoValueVec for Value {
    #[inline]
    fn into_value_vec(self) -> Vec<Value> {
        vec![self]
    }
}

impl IntoValueVec for Vec<Value> {
    #[inline]
    fn into_value_vec(self) -> Vec<Value> {
        self
    }
}

impl Deserializer {
    pub fn new<V>(values: V) -> Deserializer
        where V: IntoValueVec
    {
        Deserializer {
            values: values.into_value_vec().into_iter().peekable(),
        }
    }

    /// Returns a reference to the next value
    #[inline]
    pub fn peek(&mut self) -> Option<&Value> {
        self.values.peek()
    }

    /// Return the next value
    #[inline]
    pub fn next(&mut self) -> Result<Value> {
        match self.values.next() {
            Some(value) => Ok(value),
            None => Err(Error::EndOfStream)
        }
    }

    pub fn next_bulk(&mut self) -> Result<Vec<Value>> {
        match self.next()? {
            Value::Bulk(values) => Ok(values),
            v @ _ => Err(Error::wrong_value(format!("expected bulk but got {:?}", v)))
        }
    }

    pub fn next_bytes(&mut self) -> Result<Vec<u8>> {
        match self.next()? {
            Value::Data(bytes) => Ok(bytes),
            v => {
                let msg = format!("Expected bytes, but got {:?}", v);
                return Err(Error::wrong_value(msg));
            }
        }
    }

    pub fn read_string(&mut self) -> Result<String> {
        let redis_value = self.next()?;
        Ok(match redis_value {
            Value::Data(bytes) => String::from_utf8(bytes)?,
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
            where V: de::Visitor<'de>,
        {

            let redis_value = self.next()?;
            let value = match redis_value {
                Value::Data(bytes) => {
                    let s = String::from_utf8(bytes)?;
                    s.parse::<$ty>()?
                },
                Value::Int(i) => i as $ty,
                _ => {
                    let msg = format!("Expected Data or Int, got {:?}", &redis_value);
                    return Err(Error::wrong_value(msg));
                }
            };

            visitor.$visitor_method(value)
        }
    }
}

macro_rules! default_deserialize {
    ($($name:ident)*) => {
        $(
            #[inline]
            fn $name<V>(self, visitor: V) -> Result<V::Value>
                where V: de::Visitor<'de>
            {
                self.deserialize_any(visitor)
            }
        )*
    }
}

impl<'de> serde::Deserializer<'de> for Deserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        let s = self.read_string()?;
        visitor.visit_str(&s[..])
    }

    #[inline]
    fn deserialize_string<V>(mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        let s = self.read_string()?;
        visitor.visit_string(s)
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
        deserialize_str
        deserialize_char
        deserialize_unit
    );

    #[inline]
    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        let s = self.read_string()?;

        let b = match s.as_str() {
            "1" | "true" | "True" => true,
            "0" | "false" | "False" => false,
            _ => return Err(Error::WrongValue(format!("Expected 1/0/true/false/True/False, got {}", s)))
        };

        visitor.visit_bool(b)
    }

    #[inline]
    fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        let bytes = self.next_bytes()?;
        visitor.visit_bytes(&bytes)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(self,
                                   _name: &'static str,
                                   len: usize,
                                   visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        let values = self.next_bulk()?;
        visitor.visit_seq(SeqVisitor { iter: values.into_iter() })
    }

    #[inline]
    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        let values = self.next_bulk()?;
        visitor.visit_map(MapVisitor { iter: values.into_iter() })
    }

    #[inline]
    fn deserialize_unit_struct<V>(self,
                                  _name: &'static str,
                                  visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_ignored_any<V>(mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        let s = self.read_string()?;
        visitor.visit_str(&s[..])
    }

    #[inline]
    fn deserialize_enum<V>(mut self,
                           _enum: &'static str,
                           _variants: &'static [&'static str],
                           visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        visitor.visit_enum(EnumVisitor {
            variant: self.next()?,
            content: Value::Nil
        })
    }

    #[inline]
    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        let maybe = match self.peek() {
            Some(v) => {
                match *v {
                    Value::Data(_) => Some(()),
                    Value::Int(_) => Some(()),
                    Value::Nil => None,
                    _ => {
                        let msg = format!("Expected Data, Int, or Nil");
                        return Err(Error::wrong_value(msg));
                    }
                }
            }
            None => None,
        };

        if maybe.is_some() {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self,
                                     _name: &'static str,
                                     visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de> {
        self.deserialize_str(visitor)
    }
}

struct SeqVisitor {
    iter: vec::IntoIter<Value>,
}

impl<'de> de::SeqAccess<'de> for SeqVisitor {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: de::DeserializeSeed<'de>
    {
        match self.iter.next() {
            Some(v) => seed.deserialize(Deserializer::new(v)).map(Some),
            None => Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

struct MapVisitor {
    iter: vec::IntoIter<Value>,
}

impl<'de> serde::de::MapAccess<'de> for MapVisitor {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(v) => seed.deserialize(Deserializer::new(v)).map(Some),
            None => Ok(None)
        }
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(v) => seed.deserialize(Deserializer::new(v)),
            None => Err(Error::EndOfStream),
        }
    }
}

struct VariantVisitor {
    value: Value,
}

impl<'de> serde::de::VariantAccess<'de> for VariantVisitor {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
        where T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(Deserializer::new(self.value))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        use serde::Deserializer;
        let deserializer = self::Deserializer::new(self.value);
        deserializer.deserialize_any(visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        use serde::Deserializer;
        let deserializer = self::Deserializer::new(self.value);
        deserializer.deserialize_any(visitor)
    }
}

struct EnumVisitor {
    variant: Value,
    content: Value,
}

impl<'de> de::EnumAccess<'de> for EnumVisitor {
    type Error = Error;
    type Variant = VariantVisitor;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
        where V: de::DeserializeSeed<'de>
    {
        Ok((
            seed.deserialize(Deserializer::new(self.variant))?,
            VariantVisitor { value: self.content }
        ))
    }
}
