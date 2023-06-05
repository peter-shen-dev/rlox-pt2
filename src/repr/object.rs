use super::alloc;
use super::function::{ObjClosure, ObjFunction};
use super::native_function::NativeFunction;

use super::try_as::TryAs;
use super::{string::UnsafeString, valid::ValidPtr};
use std::fmt::Display;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Object {
    inner: ValidPtr<ObjectInner>,
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.inner.as_ref() == other.inner.as_ref()
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.as_ref().kind.fmt(f)
    }
}

impl<T: Into<ObjectKind>> From<T> for Object {
    fn from(value: T) -> Self {
        Self::from_inner(ObjectInner { kind: value.into() })
    }
}

impl Object {
    fn from_inner(inner: ObjectInner) -> Self {
        Self {
            inner: ValidPtr::from(Box::new(inner)),
        }
    }

    pub fn typename(self) -> &'static str {
        self.inner.as_ref().kind.typename()
    }

    pub unsafe fn free(self) {
        alloc::trace!("Freeing {self}");
        self.inner.as_ref().kind.free();
        self.inner.free();
    }

    pub fn kind(self) -> ObjectKind {
        self.inner.as_ref().kind
    }
}

impl<T> TryAs<T> for Object
where
    ObjectKind: TryAs<T>,
{
    fn try_as(self) -> Option<T> {
        self.kind().try_as()
    }
}

// ==================================================== Internals below here

#[derive(Copy, Clone, Debug, PartialEq)]
struct ObjectInner {
    kind: ObjectKind,
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ObjectKind {
    String { str: UnsafeString },
    Function { fun: ObjFunction },
    Closure { fun: ObjClosure },
    NativeFunction { fun: NativeFunction },
}

impl Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String { str } => str.fmt(f),
            Self::Function { fun } => write!(f, "<function {}>", fun.name),
            Self::Closure { fun } => write!(f, "<function {}>", fun.function.name),
            Self::NativeFunction { fun } => fun.fmt(f),
        }
    }
}

impl From<ObjClosure> for ObjectKind {
    fn from(fun: ObjClosure) -> Self {
        Self::Closure { fun }
    }
}

impl From<UnsafeString> for ObjectKind {
    fn from(str: UnsafeString) -> Self {
        ObjectKind::String { str }
    }
}

impl From<ObjFunction> for ObjectKind {
    fn from(fun: ObjFunction) -> Self {
        ObjectKind::Function { fun }
    }
}

impl From<NativeFunction> for ObjectKind {
    fn from(fun: NativeFunction) -> Self {
        ObjectKind::NativeFunction { fun }
    }
}

impl From<String> for ObjectKind {
    fn from(value: String) -> Self {
        ObjectKind::String {
            str: UnsafeString::from(value),
        }
    }
}

impl TryAs<ObjFunction> for ObjectKind {
    fn try_as(self) -> Option<ObjFunction> {
        match self {
            ObjectKind::Function { fun } => Some(fun),
            _ => None,
        }
    }
}

impl TryAs<UnsafeString> for ObjectKind {
    fn try_as(self) -> Option<UnsafeString> {
        match self {
            ObjectKind::String { str } => Some(str),
            _ => None,
        }
    }
}

impl ObjectKind {
    fn typename(self) -> &'static str {
        match self {
            Self::String { .. } => "string",
            Self::Closure { .. } | Self::Function { .. } => "function",
            Self::NativeFunction { .. } => "native-function",
        }
    }

    unsafe fn free(self) {
        match self {
            Self::String { str } => str.free(),
            Self::Function { fun } => fun.free(),
            Self::Closure { fun } => fun.free(),
            Self::NativeFunction { fun } => fun.free(),
        }
    }
}
