use std::rc::Rc;

use im::HashSet;
use proptest::{strategy::BoxedStrategy, strategy::Strategy};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, arbitrary::Arbitrary, proptest_derive::Arbitrary)]
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
        let depth = (if length == 0 { 0 } else { length.ilog2() }).min(4);
        Self::arbitrary_of_depth(depth, HashSet::new(), unstructured)
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        arbitrary::size_hint::recursion_guard(depth, |depth| {
            let mut choices: Vec<(usize, Option<usize>)> =
                vec![Primitive::size_hint(depth), Identifier::size_hint(depth)];
            // don't go further than 4 deep
            if depth < 4 {
                choices.push(arbitrary::size_hint::and_all(&[
                    Expr::size_hint(depth),
                    Operation::size_hint(depth),
                    Expr::size_hint(depth),
                ]));
                choices.push(arbitrary::size_hint::and_all(&[
                    Identifier::size_hint(depth),
                    Expr::size_hint(depth),
                    Expr::size_hint(depth),
                ]));
            }

            arbitrary::size_hint::or_all(&choices)
        })
    }
}

type ExprChoice<'a> = dyn FnOnce(
    &mut arbitrary::Unstructured<'a>,
    HashSet<Identifier>,
) -> arbitrary::Result<Expr<()>>;

impl<'a> Expr<()> {
    fn arbitrary_of_depth(
        depth: u32,
        bound_identifiers: HashSet<Identifier>,
        unstructured: &mut arbitrary::Unstructured<'a>,
    ) -> std::result::Result<Self, arbitrary::Error> {
        let mut choices: Vec<Box<ExprChoice<'a>>> = Vec::new();
        choices.push(Box::new(|u, _| {
            let primitive = u.arbitrary::<Primitive>()?;
            Ok(Expr::Primitive {
                annotation: (),
                value: primitive,
            })
        }));
        if !bound_identifiers.is_empty() {
            choices.push(Box::new(|u, bound| {
                let index = u.choose_index(bound.len())?;
                let name = bound.iter().nth(index).unwrap().clone();
                Ok(Expr::Identifier {
                    annotation: (),
                    name,
                })
            }));
        }
        if depth > 0 {
            choices.push(Box::new(move |u, bound| {
                let operation = u.arbitrary::<Operation>()?;
                let left = Self::arbitrary_of_depth(depth - 1, bound.clone(), u)?;
                let right = Self::arbitrary_of_depth(depth - 1, bound, u)?;
                Ok(Expr::Infix {
                    annotation: (),
                    operation,
                    left: left.into(),
                    right: right.into(),
                })
            }));
            choices.push(Box::new(move |u, bound| {
                let mut name = u.arbitrary::<Identifier>()?;
                while bound.contains(&name) {
                    name = u.arbitrary::<Identifier>()?;
                }
                let value = Self::arbitrary_of_depth(depth - 1, bound.clone(), u)?;
                let inner = Self::arbitrary_of_depth(depth - 1, bound.update(name.clone()), u)?;
                Ok(Expr::Let {
                    annotation: (),
                    name,
                    value: value.into(),
                    inner: inner.into(),
                })
            }));
        }

        let choice = unstructured.choose_index(choices.len())?;
        choices.swap_remove(choice)(unstructured, bound_identifiers)
    }
}

impl Expr<()> {
    pub fn arbitrary() -> impl Strategy<Value = Expr<()>> {
        Self::gen(0..8)
    }

    pub fn gen(depth: std::ops::Range<usize>) -> impl Strategy<Value = Expr<()>> {
        Self::gen_nested(depth, HashSet::new())
    }

    fn gen_nested(
        depth: std::ops::Range<usize>,
        bound_identifiers: HashSet<Identifier>,
    ) -> impl Strategy<Value = Expr<()>> {
        let mut choices: Vec<BoxedStrategy<Expr<()>>> = Vec::new();

        if depth.start == 0 {
            choices.push(
                Primitive::arbitrary()
                    .prop_map(|value| Expr::Primitive {
                        annotation: (),
                        value,
                    })
                    .boxed(),
            );

            if !bound_identifiers.is_empty() {
                let bound = bound_identifiers.clone();
                choices.push(
                    proptest::arbitrary::any::<proptest::sample::Index>()
                        .prop_map(move |index| Expr::Identifier {
                            annotation: (),
                            name: bound.iter().nth(index.index(bound.len())).unwrap().clone(),
                        })
                        .boxed(),
                );
            }
        }

        if depth.end > 0 {
            let next_start = if depth.start == 0 { 0 } else { depth.start - 1 };
            let next_end = depth.end - 1;

            choices.push({
                let bound = bound_identifiers.clone();
                proptest::arbitrary::any::<Operation>()
                    .prop_flat_map(move |operation| {
                        (
                            Self::gen_nested(next_start..next_end, bound.clone()),
                            Self::gen_nested(next_start..next_end, bound.clone()),
                        )
                            .prop_map(move |(left, right)| Expr::Infix {
                                annotation: (),
                                operation,
                                left: left.into(),
                                right: right.into(),
                            })
                    })
                    .boxed()
            });

            choices.push({
                let bound = bound_identifiers;
                Identifier::arbitrary_of_max_length(16)
                    .prop_flat_map(move |name| {
                        let gen_value = Self::gen_nested(next_start..next_end, bound.clone());
                        let gen_inner =
                            Self::gen_nested(next_start..next_end, bound.update(name.clone()));
                        (gen_value, gen_inner).prop_map(move |(value, inner)| Expr::Let {
                            annotation: (),
                            name: name.clone(),
                            value: value.into(),
                            inner: inner.into(),
                        })
                    })
                    .boxed()
            });
        }

        proptest::strategy::Union::new(choices)
    }
}
