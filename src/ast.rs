use std::rc::Rc;

use im::HashSet;

use crate::identifier::Identifier;
use crate::primitive::Primitive;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<Annotation> {
    Primitive {
        annotation: Annotation,
        value: Primitive,
    },
    Identifier {
        annotation: Annotation,
        name: Identifier,
    },
    Let {
        annotation: Annotation,
        name: Identifier,
        value: Rc<Expr<Annotation>>,
        inner: Rc<Expr<Annotation>>,
    },
    Infix {
        annotation: Annotation,
        operation: Operation,
        left: Rc<Expr<Annotation>>,
        right: Rc<Expr<Annotation>>,
    },
}

impl<Annotation> Expr<Annotation> {
    pub fn annotation(&self) -> &Annotation {
        match self {
            Expr::Primitive { annotation, .. } => annotation,
            Expr::Identifier { annotation, .. } => annotation,
            Expr::Let { annotation, .. } => annotation,
            Expr::Infix { annotation, .. } => annotation,
        }
    }
}

impl<Annotation> std::fmt::Display for Expr<Annotation> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, arbitrary::Arbitrary)]
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

impl<'a> arbitrary::Arbitrary<'a> for Expr<()> {
    fn arbitrary(
        unstructured: &mut arbitrary::Unstructured<'a>,
    ) -> std::result::Result<Self, arbitrary::Error> {
        let length = unstructured.arbitrary_len::<Expr<()>>()?;
        let depth = if length <= 1 {
            0
        } else {
            (usize::ilog2(length) - 1) / 4
        };
        Self::arbitrary_of_depth(depth, HashSet::new(), unstructured)
    }
}

impl<'a> Expr<()> {
    fn arbitrary_of_depth(
        depth: u32,
        bound_identifiers: HashSet<Identifier>,
        unstructured: &mut arbitrary::Unstructured<'a>,
    ) -> std::result::Result<Self, arbitrary::Error> {
        if depth == 0 {
            match (
                bound_identifiers.is_empty(),
                unstructured.int_in_range(0..=1)?,
            ) {
                (_, 0) | (true, _) => {
                    let primitive = unstructured.arbitrary::<Primitive>()?;
                    Ok(Expr::Primitive {
                        annotation: (),
                        value: primitive,
                    })
                }
                (false, 1) => {
                    let index = unstructured.choose_index(bound_identifiers.len())?;
                    let name = bound_identifiers.iter().nth(index).unwrap().clone();
                    Ok(Expr::Identifier {
                        annotation: (),
                        name,
                    })
                }
                _ => unreachable!(),
            }
        } else {
            let choice = unstructured.int_in_range(0..=1)?;
            match choice {
                0 => {
                    let operation = unstructured.arbitrary::<Operation>()?;
                    let left = Self::arbitrary_of_depth(
                        depth - 1,
                        bound_identifiers.clone(),
                        unstructured,
                    )?;
                    let right =
                        Self::arbitrary_of_depth(depth - 1, bound_identifiers, unstructured)?;
                    Ok(Expr::Infix {
                        annotation: (),
                        operation,
                        left: left.into(),
                        right: right.into(),
                    })
                }
                1 => {
                    let mut name = unstructured.arbitrary::<Identifier>()?;
                    while bound_identifiers.contains(&name) {
                        name = unstructured.arbitrary::<Identifier>()?;
                    }
                    let value = Self::arbitrary_of_depth(
                        depth - 1,
                        bound_identifiers.clone(),
                        unstructured,
                    )?;
                    let inner = Self::arbitrary_of_depth(
                        depth - 1,
                        bound_identifiers.update(name.clone()),
                        unstructured,
                    )?;
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
