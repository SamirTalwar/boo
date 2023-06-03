//! A representation of a value's type, for type-checking and valid program
//! synthesis.

/// The set of possible types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Integer,
    Function {
        parameter: Option<Box<Type>>,
        body: Option<Box<Type>>,
    },
}
