use redis::Value;
use serde::{self, ser, Serialize};
use std::fmt::{self, Display};
use std::{error, str};

pub struct Serializer;

/// Error that can be produced during serialization
#[derive(Debug)]
pub enum Error {
    Custom(String),
}

impl Error {}

pub type Result<T> = ::std::result::Result<T, Error>;

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
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
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

macro_rules! impl_num {
    ($ty:ty, $serialize_method:ident) => {
        #[inline]
        fn $serialize_method(self, v: $ty) -> Result<Value> {
            Ok(Value::Data(v.to_string().as_bytes().to_vec()))
        }
    };
}

impl<'a> serde::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeVec;
    type SerializeMap = SerializeVec;
    type SerializeStruct = SerializeVec;
    type SerializeStructVariant = SerializeVec;

    impl_num!(u8, serialize_u8);
    impl_num!(u16, serialize_u16);
    impl_num!(u32, serialize_u32);
    impl_num!(u64, serialize_u64);

    impl_num!(i8, serialize_i8);
    impl_num!(i16, serialize_i16);
    impl_num!(i32, serialize_i32);
    impl_num!(i64, serialize_i64);

    impl_num!(f32, serialize_f32);
    impl_num!(f64, serialize_f64);

    fn serialize_bool(self, v: bool) -> Result<Value> {
        match v {
            true => self.serialize_i32(1),
            false => self.serialize_i32(0),
        }
    }

    fn serialize_char(self, v: char) -> Result<Value> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(Value::Data(v.as_bytes().to_vec()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        Ok(Value::Data(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Value> {
        Ok(Value::Nil)
    }

    fn serialize_some<T>(self, v: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Nil)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }
}

pub struct SerializeVec {
    vec: Vec<Value>,
}

impl SerializeVec {
    fn to_value(&self) -> Value {
        let buffer = self.vec.to_owned();
        Value::Bulk(buffer)
    }
}

impl<'a> ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(Serializer)?;
        self.vec.push(v);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(self.to_value())
    }
}

impl<'a> ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(Serializer)?;
        self.vec.push(v);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(self.to_value())
    }
}

impl<'a> ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(Serializer)?;
        self.vec.push(v);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(self.to_value())
    }
}

impl<'a> ser::SerializeTupleVariant for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(Serializer)?;
        self.vec.push(v);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(self.to_value())
    }
}

impl<'a> ser::SerializeMap for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let v = key.serialize(Serializer)?;
        self.vec.push(v);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(Serializer)?;
        self.vec.push(v);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(self.to_value())
    }
}

impl<'a> ser::SerializeStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let k = Value::Data(key.as_bytes().to_vec());
        let v = value.serialize(Serializer)?;
        self.vec.push(k);
        self.vec.push(v);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(self.to_value())
    }
}

impl<'a> ser::SerializeStructVariant for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let k = Value::Data(key.as_bytes().to_vec());
        let v = value.serialize(Serializer)?;
        self.vec.push(k);
        self.vec.push(v);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(self.to_value())
    }
}
