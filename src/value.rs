use crate::error::RuntimeError;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Value {
    Int(u32),
    Str(String),
}

impl Value {
    #[inline]
    pub fn into_bool(&self) -> bool {
        self.into()
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(..) => "int",
            Value::Str(..) => "str",
        }
    }
}

impl Display for Value {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Value::Int(num) => num.fmt(formatter),
            Value::Str(str) => formatter.write_str(str),
        }
    }
}

impl From<bool> for Value {
    #[inline]
    fn from(b: bool) -> Self {
        if b {
            Value::Int(1)
        } else {
            Value::Int(0)
        }
    }
}

impl<'a> From<Value> for bool {
    #[inline]
    fn from(v: Value) -> Self {
        match v {
            Value::Int(i) => i != 0,
            Value::Str(s) => !s.is_empty(),
        }
    }
}

impl<'a> From<&'a Value> for bool {
    #[inline]
    fn from(v: &'a Value) -> Self {
        match v {
            Value::Int(i) => *i != 0,
            Value::Str(s) => !s.is_empty(),
        }
    }
}

impl From<u32> for Value {
    #[inline]
    fn from(n: u32) -> Self {
        Value::Int(n)
    }
}

impl From<String> for Value {
    #[inline]
    fn from(s: String) -> Self {
        Value::Str(s)
    }
}

impl<'a> From<&'a str> for Value {
    #[inline]
    fn from(s: &'a str) -> Self {
        Value::Str(s.to_string())
    }
}

impl TryFrom<Value> for u32 {
    type Error = RuntimeError;

    #[inline]
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Int(n) => Ok(n),
            _ => Err(RuntimeError::TypeError(v.type_name())),
        }
    }
}

impl TryFrom<Value> for usize {
    type Error = RuntimeError;

    #[inline]
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Int(n) => Ok(n as usize),
            _ => Err(RuntimeError::TypeError(v.type_name())),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = RuntimeError;

    #[inline]
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Str(s) => Ok(s),
            _ => Err(RuntimeError::TypeError(v.type_name())),
        }
    }
}
