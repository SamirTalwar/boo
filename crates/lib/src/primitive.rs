pub mod integer;

use proptest::strategy::Strategy;

use crate::types::Type;

pub use integer::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Primitive {
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
    pub fn get_type(&self) -> Type {
        match self {
            Self::Integer(_) => Type::Integer,
        }
    }

    pub fn arbitrary() -> impl Strategy<Value = Primitive> {
        Integer::arbitrary().prop_map(Primitive::Integer)
    }

    pub fn arbitrary_of_type(target_type: Type) -> Option<impl Strategy<Value = Primitive>> {
        match target_type {
            Type::Integer => Some(Integer::arbitrary().prop_map(Primitive::Integer)),
            _ => None,
        }
    }
}
