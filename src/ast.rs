use std::rc::Rc;

use crate::identifier::Identifier;
use crate::primitive::Primitive;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<'a, Annotation> {
    Primitive {
        annotation: Annotation,
        value: Primitive,
    },
    Identifier {
        annotation: Annotation,
        name: Identifier<'a>,
    },
    Let {
        annotation: Annotation,
        name: Identifier<'a>,
        value: Rc<Expr<'a, Annotation>>,
        inner: Rc<Expr<'a, Annotation>>,
    },
    Infix {
        annotation: Annotation,
        operation: Operation,
        left: Rc<Expr<'a, Annotation>>,
        right: Rc<Expr<'a, Annotation>>,
    },
}

impl<'a, Annotation> Expr<'a, Annotation> {
    pub fn annotation(&self) -> &Annotation {
        match self {
            Expr::Primitive { annotation, .. } => annotation,
            Expr::Identifier { annotation, .. } => annotation,
            Expr::Let { annotation, .. } => annotation,
            Expr::Infix { annotation, .. } => annotation,
        }
    }
}

impl<'a, Annotation> std::fmt::Display for Expr<'a, Annotation> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Primitive {
                annotation: _,
                value,
            } => value.fmt(f),
            Expr::Identifier {
                annotation: _,
                name,
            } => name.fmt(f),
            Expr::Let {
                annotation: _,
                name,
                value,
                inner,
            } => write!(f, "let {} = ({}) in ({})", name, value, inner),
            Expr::Infix {
                annotation: _,
                operation,
                left,
                right,
            } => write!(f, "({}) {} ({})", left, operation, right),
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

    impl<'a> arbitrary::Arbitrary<'a> for Expr<'a, ()> {
        fn arbitrary(
            unstructured: &mut arbitrary::Unstructured<'a>,
        ) -> std::result::Result<Self, arbitrary::Error> {
            let length = unstructured.arbitrary_len::<Expr<()>>()?;
            let depth = if length == 0 { 1 } else { usize::ilog2(length) };
            Self::arbitrary_of_depth(depth, unstructured)
        }
    }

    impl<'a> Expr<'a, ()> {
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
                let choice = unstructured.int_in_range(0..=1)?;
                match choice {
                    0 => {
                        let operation = unstructured.arbitrary::<Operation>()?;
                        let left = Self::arbitrary_of_depth(depth - 1, unstructured)?;
                        let right = Self::arbitrary_of_depth(depth - 1, unstructured)?;
                        Ok(Expr::Infix {
                            annotation: (),
                            operation,
                            left: left.into(),
                            right: right.into(),
                        })
                    }
                    1 => {
                        let name = unstructured.arbitrary::<Identifier>()?;
                        let value = Self::arbitrary_of_depth(depth - 1, unstructured)?;
                        let inner = Self::arbitrary_of_depth(depth - 1, unstructured)?;
                        Ok(Expr::Let {
                            annotation: (),
                            name,
                            value: value.into(),
                            inner: inner.into(),
                        })
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
