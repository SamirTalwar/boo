#[derive(Debug, Clone, Copy, PartialEq, Eq, proptest_derive::Arbitrary)]
pub enum Operation {
    Add,
    Subtract,
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
