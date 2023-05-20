use std::{fmt::Display};

#[cfg(feature = "verbose_allocations")]
use tracing::trace;
#[cfg(not(feature = "verbose_allocations"))]
use crate::noop as trace;

/// SAFETY: The presiding assumption here is that we basically just got this from Box or String or whatever and it's okay to use,
///     but it needs to be a raw pointer so we can share it and garbage collect efficiently
type ValidPtr<T> = std::ptr::NonNull<T>;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Object {
    object: ValidPtr<ObjectInner>,
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.object.as_ref() == other.object.as_ref() }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { self.object.as_ref().kind.fmt(f) }
    }
}

impl Object {
    pub fn typename(&self) -> &'static str {
        unsafe { self.object.as_ref().kind.typename() }
    }

    fn from_inner(kind: ObjectKind) -> Object {
        let object = Box::leak(Box::new(ObjectInner { kind }));
        unsafe {
            Object {
                object: ValidPtr::new_unchecked(object as *mut _),
            }
        }
    }

    pub fn make_str(value: String) -> Object {
        trace!("Allocating string '{value}'");
        let str = ObjectKind::from(value);
        Self::from_inner(str)
    }

    pub fn is_string(&self) -> bool {
        let inner = unsafe { self.object.as_ref() };
        matches!(inner.kind, ObjectKind::String { .. })
    }

    pub fn concatenate(&self, other: &Self) -> Self {
        let (lhs, rhs) = unsafe { (self.object.as_ref().kind, other.object.as_ref().kind) };
        let (ObjectKind::String { str: lhs }, ObjectKind::String {str: rhs}) = (lhs, rhs) else {
            unreachable!("TODO: This is scuffed, but it's a slight defensive measure");
        };
        Object::make_str(unsafe { String::from(lhs.as_ref()) + rhs.as_ref() })
    }

    pub unsafe fn free(&self) {
        trace!("Freeing {self}");
        self.object.as_ref().kind.free();
        drop(Box::from_raw(self.object.as_ptr()));
    }

    pub fn compare_str(&self, s: &str) -> bool {
        let inner = unsafe { self.object.as_ref() };
        matches!(inner.kind, ObjectKind::String { str } if unsafe { str.as_ref() } == s)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct ObjectInner {
    kind: ObjectKind
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
enum ObjectKind {
    // ! If mutability is ever added, many of these "as_ref" may become suspicious (as far as a safe API goes)
    String { str: ValidPtr<str> },
}

impl PartialEq for ObjectKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjectKind::String { str: a }, ObjectKind::String { str: b }) => {
                unsafe {
                    // SAFETY: These are always valid, and only take a shared reference
                    a.as_ref() == b.as_ref()
                }
            }
        }
    }
}

impl Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String { str } => unsafe { write!(f, "\"{}\"", str.as_ref()) },
        }
    }
}

impl From<String> for ObjectKind {
    fn from(value: String) -> Self {
        let boxed = value.into_boxed_str();
        let str = unsafe { ValidPtr::new_unchecked(Box::leak(boxed) as *mut _) };
        ObjectKind::String { str }
    }
}

impl ObjectKind {
    fn typename(&self) -> &'static str {
        match self {
            Self::String { .. } => "string",
        }
    }

    unsafe fn free(&self) {
        match self {
            Self::String { str } => {
                unsafe {
                    drop(Box::from_raw(str.as_ptr()));
                }
            }
        }
    }
}
