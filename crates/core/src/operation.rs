//! Infix operations on integers.

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

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Add => write!(f, "+"),
            Operation::Subtract => write!(f, "-"),
            Operation::Multiply => write!(f, "*"),
        }
    }
}
