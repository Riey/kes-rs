use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Value<'b> {
    Int(u32),
    Str(&'b str),
}

impl<'b> Value<'b> {
    #[inline]
    pub fn into_bool(self) -> bool {
        self.into()
    }

    pub fn type_name(self) -> &'static str {
        match self {
            Value::Int(..) => "int",
            Value::Str(..) => "str",
        }
    }
}

impl<'b> Display for Value<'b> {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Value::Int(num) => num.fmt(formatter),
            Value::Str(str) => formatter.write_str(str),
        }
    }
}

impl<'b> From<bool> for Value<'b> {
    #[inline]
    fn from(b: bool) -> Self {
        if b {
            Value::Int(1)
        } else {
            Value::Int(0)
        }
    }
}

impl<'b> From<Value<'b>> for bool {
    #[inline]
    fn from(v: Value) -> Self {
        match v {
            Value::Int(0) | Value::Str("") => false,
            _ => true,
        }
    }
}

impl<'b> From<u32> for Value<'b> {
    #[inline]
    fn from(n: u32) -> Self {
        Value::Int(n)
    }
}

impl<'b> From<&'b str> for Value<'b> {
    #[inline]
    fn from(s: &'b str) -> Self {
        Value::Str(s)
    }
}

impl<'b> From<&'b mut str> for Value<'b> {
    #[inline]
    fn from(s: &'b mut str) -> Self {
        Value::Str(s)
    }
}

/// Contains actual type name
#[derive(Debug)]
pub struct ValueConvertError(pub &'static str);

impl<'b> TryFrom<Value<'b>> for u32 {
    type Error = ValueConvertError;

    #[inline]
    fn try_from(v: Value<'b>) -> Result<Self, Self::Error> {
        match v {
            Value::Int(n) => Ok(n),
            _ => Err(ValueConvertError(v.type_name())),
        }
    }
}

impl<'b> TryFrom<Value<'b>> for usize {
    type Error = ValueConvertError;

    #[inline]
    fn try_from(v: Value<'b>) -> Result<Self, Self::Error> {
        match v {
            Value::Int(n) => Ok(n as usize),
            _ => Err(ValueConvertError(v.type_name())),
        }
    }
}

impl<'b> TryFrom<Value<'b>> for &'b str {
    type Error = ValueConvertError;

    #[inline]
    fn try_from(v: Value<'b>) -> Result<Self, Self::Error> {
        match v {
            Value::Str(s) => Ok(s),
            _ => Err(ValueConvertError(v.type_name())),
        }
    }
}
