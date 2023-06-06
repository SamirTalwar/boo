//! A representation of a value's type, for type-checking and valid program
//! synthesis.

use std::rc::Rc;

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
    Integer,
    Function { parameter: Type, body: Type },
}
