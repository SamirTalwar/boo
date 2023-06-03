//! Primitive values.

pub mod integer;

use proptest::strategy::Strategy;

use crate::types::Type;

pub use integer::*;

/// The set of valid primitive values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Primitive {
    /// An [`Integer`] value.
    Integer(Integer),
}

impl std::fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Integer(value) => write!(f, "{}", value),
        }
    }
}

impl Primitive {
    /// Gets the type of a primitive.
    pub fn get_type(&self) -> Type {
        match self {
            Self::Integer(_) => Type::Integer,
        }
    }

    /// A proptest strategy for an arbitrary primitive value.
    pub fn arbitrary() -> impl Strategy<Value = Primitive> {
        Integer::arbitrary().prop_map(Primitive::Integer)
    }

    /// A proptest strategy for an arbitrary primitive value of the given type.
    ///
    /// Returns `None` if there is no primitive matching the given type.
    pub fn arbitrary_of_type(target_type: Type) -> Option<impl Strategy<Value = Primitive>> {
        match target_type {
            Type::Integer => Some(Integer::arbitrary().prop_map(Primitive::Integer)),
            _ => None,
        }
    }
}
