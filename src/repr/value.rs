use super::{alloc, object::ObjectKind};
use std::fmt::Display;

use super::object::Object;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Num(f64),
    Bool(bool),
    Nil,
    Object(Object),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Num(n) => n.fmt(f),
            Self::Bool(b) => b.fmt(f),
            Self::Nil => write!(f, "nil"),
            Self::Object(obj) => obj.fmt(f),
        }
    }
}

impl Value {
    pub fn typename(&self) -> &'static str {
        match self {
            Self::Bool(_) => "boolean",
            Self::Num(_) => "number",
            Self::Nil => "nil",
            Self::Object(obj) => obj.typename(),
        }
    }

    pub fn falsey(&self) -> bool {
        matches!(self, Self::Bool(false) | Self::Nil)
    }

    pub fn try_as<T: TryFrom<ObjectKind>>(self) -> Option<T> {
        match self {
            Self::Object(obj) => obj.try_as::<T>(),
            _ => None,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Num(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        alloc::trace!("Allocating string '{value}'");
        Self::Object(Object::from(String::from(value)))
    }
}

impl<T: Into<Object>> From<T> for Value {
    fn from(value: T) -> Self {
        Value::Object(value.into())
    }
}
