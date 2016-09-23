use std::error;
use std::fmt;
use std::iter::Peekable;
use std::num;
use std::string;
use std::vec;

use redis::Value;

use serde::{self, de};
use serde::de::Visitor;

/// Error that can be produced during deserialization
#[derive(Debug)]
pub enum Error {
    Custom(String),
    TypeMismatch(String),
    EndOfStream,
    UnknownField(String),
    MissingField(&'static str),
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
            Error::TypeMismatch(_) => "type mismatch when decoding redis values",
            Error::EndOfStream => "end of redis value stream",
            Error::UnknownField(_) => "unknown field",
            Error::MissingField(_) => "missing field",
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
            Error::TypeMismatch(ref t) => write!(f, "TypeMismatch(expected: {})", t),
            Error::EndOfStream => write!(f, "Reached end of stream"),
            Error::UnknownField(ref field) => write!(f, "got unexpected field: {}", field),
            Error::MissingField(ref field) => write!(f, "missing field: {}", field),
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
    fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Custom(msg.into())
    }

    /// Raised when a `Deserialize` type unexpectedly hit the end of the stream.
    fn end_of_stream() -> Self {
        Error::EndOfStream
    }

    /// Raised when a `Deserialize` was passed an incorrect value.
    fn invalid_value(msg: &str) -> Self {
        Error::custom(format!("Invalid value: {}", msg))
    }

    /// Raised when a fixed sized sequence or map was passed in the wrong amount of arguments.
    fn invalid_length(len: usize) -> Self {
        Error::custom(format!("Invalid length: {}", len))
    }

    /// Raised when a `Deserialize` enum type received an unexpected variant.
    fn unknown_variant(field: &str) -> Self {
        Error::custom(format!("Unknown variant `{}`", field))
    }

    fn unknown_field(field: &str) -> Error {
        Error::UnknownField(field.into())
    }

    fn missing_field(field: &'static str) -> Error {
        Error::MissingField(field)
    }

    fn invalid_type(t: ::serde::de::Type) -> Error {
        Error::TypeMismatch(format!("{:?}", t))
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
            None => Err(serde::de::Error::end_of_stream())
        }
    }

    pub fn next_bulk(&mut self) -> Result<Vec<Value>> {
        match try!(self.next()) {
            Value::Bulk(values) => Ok(values),
            v @ _ => Err(Error::wrong_value(format!("expected bulk but got {:?}", v)))
        }
    }

    pub fn read_string(&mut self) -> Result<String> {
        let redis_value = try!(self.next());
        Ok(match redis_value {
            Value::Data(bytes) => try!(String::from_utf8(bytes)),
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
        fn $deserialize_method<V>(&mut self, mut visitor: V) -> Result<V::Value>
            where V: serde::de::Visitor,
        {

            let redis_value = try!(self.next());
            let value = match redis_value {
                Value::Data(bytes) => {
                    let s = try!(String::from_utf8(bytes));
                    try!(s.parse::<$ty>())
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
            fn $name<V>(&mut self, visitor: V) -> Result<V::Value>
                where V: serde::de::Visitor
            {
                self.deserialize(visitor)
            }
        )*
    }
}

impl serde::Deserializer for Deserializer {
    type Error = Error;

    #[inline]
    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        let s = try!(self.read_string());
        visitor.visit_str(&s[..])
    }

    #[inline]
    fn deserialize_string<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        let s = try!(self.read_string());
        visitor.visit_string(s)
    }

    impl_num!(u8, deserialize_u8, visit_u8);
    impl_num!(u16, deserialize_u16, visit_u16);
    impl_num!(u32, deserialize_u32, visit_u32);
    impl_num!(u64, deserialize_u64, visit_u64);
    impl_num!(usize, deserialize_usize, visit_usize);

    impl_num!(i8, deserialize_i8, visit_i8);
    impl_num!(i16, deserialize_i16, visit_i16);
    impl_num!(i32, deserialize_i32, visit_i32);
    impl_num!(i64, deserialize_i64, visit_i64);
    impl_num!(isize, deserialize_isize, visit_isize);

    impl_num!(f32, deserialize_f32, visit_f32);
    impl_num!(f64, deserialize_f64, visit_f64);

    default_deserialize!(
        deserialize_str
        deserialize_char
        deserialize_bool
        deserialize_unit
    );

    #[inline]
    fn deserialize_bytes<V>(&mut self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(&mut self,
                                   _name: &'static str,
                                   len: usize,
                                   visitor: V) -> Result<V::Value>
        where V: Visitor,
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_tuple<V>(&mut self, _len: usize, visitor: V) -> Result<V::Value>
        where V: Visitor,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_seq<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        let values = try!(self.next_bulk());
        let mut de = Deserializer::new(values);
        visitor.visit_seq(SeqVisitor { de: &mut de })
    }

    #[inline]
    fn deserialize_seq_fixed_size<V>(&mut self, _len: usize, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_map<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        let values = try!(self.next_bulk());
        let mut de = Deserializer::new(values);
        visitor.visit_map(MapVisitor { de: &mut de })
    }

    #[inline]
    fn deserialize_unit_struct<V>(&mut self,
                                  _name: &'static str,
                                  visitor: V) -> Result<V::Value>
        where V: Visitor,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_struct<V>(&mut self,
                             _name: &'static str,
                             _fields: &'static [&'static str],
                             visitor: V) -> Result<V::Value>
        where V: Visitor,
    {
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_struct_field<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        let s = try!(self.read_string());
        visitor.visit_str(&s[..])
    }

    #[inline]
    fn deserialize_ignored_any<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        let s = try!(self.read_string());
        visitor.visit_str(&s[..])
    }

    #[inline]
    fn deserialize_enum<V>(&mut self,
                     _enum: &'static str,
                     _variants: &'static [&'static str],
                     mut visitor: V) -> Result<V::Value>
        where V: serde::de::EnumVisitor,
    {
        visitor.visit(VariantVisitor { de: self })
    }

    #[inline]
    fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
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
    fn deserialize_newtype_struct<V>(&mut self,
                                     _name: &'static str,
                                     mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_newtype_struct(self)
    }
}

struct SeqVisitor<'a> {
    de: &'a mut Deserializer,
}

impl<'a> de::SeqVisitor for SeqVisitor<'a> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: de::Deserialize
    {
        if self.de.peek().is_some() {
            let value = try!(serde::Deserialize::deserialize(self.de));
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn end(&mut self) -> Result<()> {
        Ok(())
    }
}

struct MapVisitor<'a> {
    de: &'a mut Deserializer
}

impl<'a> serde::de::MapVisitor for MapVisitor<'a> {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: de::Deserialize,
    {
        loop {
            if self.de.peek().is_some() {
                let key = try!(serde::Deserialize::deserialize(self.de));

                if self.de.peek() == Some(&Value::Data(b"".to_vec())) {
                    // Empty string value, don't do anything with it.
                    self.de.next().ok();
                    continue;
                }

                return Ok(Some(key));
            } else {
                return Ok(None);
            }
        }
    }

    #[inline]
    fn visit_value<V>(&mut self) -> Result<V>
        where V: de::Deserialize,
    {
        serde::Deserialize::deserialize(self.de)
    }

    #[inline]
    fn end(&mut self) -> Result<()> {
        // ignore any unused values since keys can randomly be added in Redis
        Ok(())
    }

    fn missing_field<V>(&mut self, _field: &'static str) -> Result<V>
        where V: de::Deserialize,
    {

        let mut de = de::value::ValueDeserializer::into_deserializer(());
        de::Deserialize::deserialize(&mut de)
    }
}

struct VariantVisitor<'a> {
    de: &'a mut Deserializer
}

impl<'a> serde::de::VariantVisitor for VariantVisitor<'a> {
    type Error = Error;

    fn visit_variant<V>(&mut self) -> Result<V>
        where V: serde::Deserialize,
    {
        let value = try!(serde::Deserialize::deserialize(self.de));
        Ok(value)
    }

    fn visit_unit(&mut self) -> Result<()> {
        Ok(())
    }

    fn visit_newtype<T>(&mut self) -> Result<T>
        where T: serde::de::Deserialize,
    {
        Err(de::Error::custom("newtype variants are not supported"))
    }

    fn visit_tuple<V>(&mut self,
                      _len: usize,
                      _visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        Err(de::Error::custom("tuple variants are not supported"))
    }

    fn visit_struct<V>(&mut self,
                       _fields: &'static [&'static str],
                       _visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        Err(de::Error::custom("struct variants are not supported"))
    }
}

