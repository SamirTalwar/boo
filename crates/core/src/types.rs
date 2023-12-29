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

/// A type bound by forall quantifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polytype {
    pub quantifiers: Vec<TypeVariable>,
    pub mono: Monotype,
}

impl From<Type<Monotype>> for Polytype {
    fn from(value: Type<Monotype>) -> Self {
        Self::from(Monotype::from(value))
    }
}

impl From<Monotype> for Polytype {
    fn from(value: Monotype) -> Self {
        Self {
            quantifiers: vec![],
            mono: value,
        }
    }
}

/// The set of types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type<Outer: TypeRef> {
    Integer,
    Function { parameter: Outer, body: Outer },
    Variable(TypeVariable),
}

impl<Outer: TypeRef> Type<Outer> {
    pub fn transform<NewOuter: TypeRef>(self, f: impl Fn(Outer) -> NewOuter) -> Type<NewOuter> {
        match self {
            Type::Integer => Type::Integer,
            Type::Function { parameter, body } => Type::Function {
                parameter: f(parameter),
                body: f(body),
            },
            Type::Variable(variable) => Type::Variable(variable),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVariable(pub Arc<String>);

impl TypeVariable {
    pub fn new(value: String) -> Self {
        Self(Arc::new(value))
    }
    pub fn new_from_str(value: &str) -> Self {
        Self::new(value.to_owned())
    }
}
