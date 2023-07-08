//! Infix operations on integers.

use crate::identifier::Identifier;

/// The set of valid infix operations on integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, proptest_derive::Arbitrary)]
pub enum Operation {
    /// Adds two integers.
    Add,
    /// Subtracts an integer from another.
    Subtract,
    /// Multiplies two integers.
    Multiply,
}

impl Operation {
    /// Construct an identifier representing this operation.
    pub fn identifier(&self) -> Identifier {
        match self {
            Operation::Add => Identifier::operator_from_str("+"),
            Operation::Subtract => Identifier::operator_from_str("-"),
            Operation::Multiply => Identifier::operator_from_str("*"),
        }
        .unwrap()
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.identifier().fmt(f)
    }
}
