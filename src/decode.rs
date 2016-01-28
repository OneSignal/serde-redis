use std::error;
use std::fmt;
use std::string;
use std::num;

use redis::Value;

use serde::{self, de};

/// Error that can be produced during deserialization
#[derive(Debug)]
pub enum Error {
    Syntax(String),
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
        unimplemented!();
    }

    fn cause(&self) -> Option<&error::Error> {
        unimplemented!();
    }
}

impl fmt::Display for Error {
    fn fmt(&self, _fmt: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!();
    }
}

impl de::Error for Error {
    fn syntax(desc: &str) -> Error {
        Error::Syntax(desc.into())
    }

    fn end_of_stream() -> Error {
        Error::EndOfStream
    }

    fn unknown_field(field: &str) -> Error {
        Error::UnknownField(field.into())
    }

    fn missing_field(field: &'static str) -> Error {
        Error::MissingField(field)
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
#[derive(Debug)]
pub struct Deserializer {
    redis_values: Vec<Value>,
}

impl Deserializer {
    pub fn new(redis_value: Value) -> Result<Deserializer> {
        let mut values = match redis_value {
            Value::Data(bytes) => vec![Value::Data(bytes)],
            Value::Bulk(bulk) => bulk,
            Value::Int(i) => vec![Value::Int(i)],
            Value::Nil => vec![Value::Nil],
            _ => return Err(Error::wrong_value("must be a redis value type"))
        };

        // TODO: would be better to turn this into an iterator than have to reverse and use the
        // vector.
        values.reverse();

        Ok(Deserializer {
            redis_values: values,
        })
    }

    pub fn peek(&mut self) -> Option<&Value> {
        self.redis_values.last()
    }

    pub fn next(&mut self) -> Result<Value> {
        match self.redis_values.pop() {
            Some(v) => Ok(v),
            None => Err(serde::de::Error::end_of_stream())
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

impl serde::Deserializer for Deserializer {
    type Error = Error;

    #[inline]
    fn deserialize<V>(&mut self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor,
    {
        Err(Error::DeserializeNotSupported)
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

    #[inline]
    fn deserialize_seq<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_seq(SeqVisitor { de: self })
    }


    #[inline]
    fn deserialize_map<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        visitor.visit_map(MapVisitor { de: self })
    }

    #[inline]
    fn deserialize_struct_key<V>(&mut self, mut visitor: V) -> Result<V::Value>
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
        if self.de.peek().is_none() {
            Ok(())
        } else {
            Err(serde::de::Error::syntax("expected end"))
        }
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
        if self.de.peek().is_some() {
            let key = try!(serde::Deserialize::deserialize(self.de));
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: de::Deserialize,
    {
        let value = try!(serde::Deserialize::deserialize(self.de));
        Ok(value)
    }

    fn end(&mut self) -> Result<()> {
        // ignore any unused values since keys can randomly be added in Redis
        Ok(())
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
        Err(de::Error::syntax("newtype variants are not supported"))
    }

    fn visit_tuple<V>(&mut self,
                      _len: usize,
                      _visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        Err(de::Error::syntax("tuple variants are not supported"))
    }

    fn visit_struct<V>(&mut self,
                       _fields: &'static [&'static str],
                       _visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        Err(de::Error::syntax("struct variants are not supported"))
    }
}

