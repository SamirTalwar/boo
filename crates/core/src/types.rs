//! A representation of a value's type.
//!
//! Used for type-checking and valid program synthesis.

use std::borrow::Borrow;
use std::fmt::Display;
use std::sync::Arc;

/// An opaque wrapper around a type.
pub trait TypeRef: From<Type<Self>> + Display + Sized {}

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

impl Display for Monotype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TypeRef for Monotype {}

/// A type bound by forall quantifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polytype {
    pub quantifiers: Vec<TypeVariable>,
    pub mono: Monotype,
}

impl Polytype {
    pub fn unquantified(mono: Monotype) -> Self {
        Polytype {
            quantifiers: vec![],
            mono,
        }
    }
}

impl Display for Polytype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.quantifiers.is_empty() {
            write!(f, "{}", self.mono)
        } else {
            write!(f, "∀")?;
            for quantifier in self.quantifiers.iter() {
                write!(f, " {quantifier}")?;
            }
            write!(f, ". ")?;
            write!(f, "{}", self.mono)
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

impl<Outer: TypeRef> Display for Type<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Integer => write!(f, "Integer"),
            Type::Function { parameter, body } => write!(f, "({parameter} -> {body})"),
            Type::Variable(variable) => write!(f, "{variable}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeVariable(pub Arc<String>);

impl TypeVariable {
    pub fn new(value: String) -> Self {
        Self(Arc::new(value))
    }
    pub fn new_from_str(value: &str) -> Self {
        Self::new(value.to_owned())
    }
}

impl Borrow<str> for TypeVariable {
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for TypeVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
