use redis::Value;
use serde::{self, ser, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Display};
use std::iter::Peekable;
use std::{error, num, str, string, vec};

pub struct Serializer {
    output: HashMap<String, Value>,
}

pub fn to_vec<T>(value: &T) -> Result<Vec<Value>>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: HashMap::new(),
    };
    value.serialize(&mut serializer)?;
    let mut vec = Vec::new();
    for (k, v) in serializer.output {
        vec.push(Value::Data(k.as_bytes().to_vec()));
        vec.push(v);
    }
    Ok(vec)
}

pub fn to_map<T>(value: &T) -> Result<HashMap<String, Value>>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: HashMap::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

/// Error that can be produced during serialization
#[derive(Debug)]
pub enum Error {
    Custom(String),
}

impl Error {}

pub type Result<T> = ::std::result::Result<T, Error>;

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Custom(ref reason) => write!(f, "CustomError({})", reason),
        }
    }
}

impl ser::Error for Error {
    /// Raised when there is general error when deserializing a type.
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl<'a> serde::Serializer for &'a mut Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Value> {
        todo!()
    }
    fn serialize_i8(self, v: i8) -> Result<Value> {
        todo!()
    }

    fn serialize_i16(self, v: i16) -> Result<Value> {
        todo!()
    }

    fn serialize_i32(self, v: i32) -> Result<Value> {
        todo!()
    }

    fn serialize_i64(self, v: i64) -> Result<Value> {
        todo!()
    }

    fn serialize_u8(self, v: u8) -> Result<Value> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> Result<Value> {
        todo!()
    }

    fn serialize_u32(self, v: u32) -> Result<Value> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> Result<Value> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<Value> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Value> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Value> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(Value::Data(v.as_bytes().to_vec()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        todo!()
    }

    fn serialize_none(self) -> Result<Value> {
        todo!()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Value> {
        todo!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        todo!()
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Value> {
        todo!()
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Value> {
        todo!()
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Value> {
        todo!()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Value> {
        todo!()
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Value> {
        todo!()
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(&mut **self)?;
        self.output.insert(key.to_string(), value);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Nil)
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Nil)
    }
}
