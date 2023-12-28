//! A representation of a value's type.
//!
//! Used for type-checking and valid program synthesis.

use std::sync::Arc;

/// An opaque wrapper around a type.
pub trait TypeRef: From<Type<Self>> + Sized {}

/// A simple type wrapper that allows for cycles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monotype(pub Arc<Type<Self>>);

impl AsRef<Type<Self>> for Monotype {
    fn as_ref(&self) -> &Type<Self> {
        self.0.as_ref()
    }
}

impl From<Type<Self>> for Monotype {
    fn from(value: Type<Self>) -> Self {
        Self(Arc::new(value))
    }
}

impl TypeRef for Monotype {}

/// The set of types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type<Outer: TypeRef> {
    Integer,
    Function { parameter: Outer, body: Outer },
}

impl<Outer: TypeRef> Type<Outer> {
    pub fn transform<NewOuter: TypeRef>(self, f: impl Fn(Outer) -> NewOuter) -> Type<NewOuter> {
        match self {
            Type::Integer => Type::Integer,
            Type::Function { parameter, body } => Type::Function {
                parameter: f(parameter),
                body: f(body),
            },
        }
    }
}
