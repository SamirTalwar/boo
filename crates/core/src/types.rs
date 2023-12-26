//! A representation of a value's type.
//!
//! Used for type-checking and valid program synthesis.

use std::sync::Arc;

/// An opaque wrapper around a type.
pub trait TypeRef: From<Type<Self>> + Sized {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcType(Arc<Type<Self>>);

impl AsRef<Type<Self>> for ArcType {
    fn as_ref(&self) -> &Type<Self> {
        self.0.as_ref()
    }
}

impl From<Type<Self>> for ArcType {
    fn from(value: Type<Self>) -> Self {
        Self(Arc::new(value))
    }
}

impl TypeRef for ArcType {}

/// The set of types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type<Outer: TypeRef> {
    Integer,
    Function { parameter: Outer, body: Outer },
}
