#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<Annotation> {
    Primitive {
        annotation: Annotation,
        value: Primitive,
    },
    Infix {
        annotation: Annotation,
        operation: Operation,
        left: Box<Expr<Annotation>>,
        right: Box<Expr<Annotation>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum Primitive {
    Int(Int),
}

pub type Int = i64;

impl std::fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Int(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
}
