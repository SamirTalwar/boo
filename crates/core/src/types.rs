//! A representation of a value's type, for type-checking and valid program
//! synthesis.

use std::rc::Rc;

use proptest::prelude::*;

/// Types can be known or unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Unknown,
    Known(Rc<KnownType>),
}

impl Type {
    pub fn is_known(&self) -> bool {
        matches!(self, Type::Known(_))
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }
}

/// The set of known types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnownType {
    Boolean,
    Integer,
    Function { parameter: Type, body: Type },
}

impl Type {
    pub fn arbitrary_primitive() -> impl Strategy<Value = KnownType> {
        prop_oneof![Just(KnownType::Boolean), Just(KnownType::Integer)]
    }
}
