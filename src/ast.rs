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

impl<Annotation> std::fmt::Display for Expr<Annotation> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Primitive {
                annotation: _,
                value,
            } => value.fmt(f),
            Expr::Infix {
                annotation: _,
                operation,
                left,
                right,
            } => write!(f, "({} {} {})", left, operation, right),
        }
    }
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

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Add => write!(f, "+"),
            Operation::Subtract => write!(f, "-"),
            Operation::Multiply => write!(f, "*"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'a> arbitrary::Arbitrary<'a> for Expr<()> {
        fn arbitrary(
            unstructured: &mut arbitrary::Unstructured<'a>,
        ) -> std::result::Result<Self, arbitrary::Error> {
            let length = unstructured.arbitrary_len::<Expr<()>>()?;
            let depth = if length == 0 { 1 } else { usize::ilog2(length) };
            Self::arbitrary_of_depth(depth, unstructured)
        }
    }

    impl<'a> Expr<()> {
        fn arbitrary_of_depth(
            depth: u32,
            unstructured: &mut arbitrary::Unstructured<'a>,
        ) -> std::result::Result<Self, arbitrary::Error> {
            if depth == 0 {
                let primitive = unstructured.arbitrary::<Primitive>()?;
                Ok(Expr::Primitive {
                    annotation: (),
                    value: primitive,
                })
            } else {
                let operation = unstructured.arbitrary::<Operation>()?;
                let left = Self::arbitrary_of_depth(depth - 1, unstructured)?;
                let right = Self::arbitrary_of_depth(depth - 1, unstructured)?;
                Ok(Expr::Infix {
                    annotation: (),
                    operation,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
        }
    }
}
