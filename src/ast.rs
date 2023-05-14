use std::rc::Rc;

use im::HashSet;
use proptest::{strategy::BoxedStrategy, strategy::Strategy};

use crate::identifier::Identifier;
use crate::primitive::Primitive;
use crate::span::Span;

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

#[derive(Debug)]
pub struct ExprGenConfig {
    pub depth: std::ops::Range<usize>,
    pub gen_identifier: Rc<BoxedStrategy<Identifier>>,
}

impl Default for ExprGenConfig {
    fn default() -> Self {
        Self {
            depth: 0..8,
            gen_identifier: Rc::new(Identifier::arbitrary().boxed()),
        }
    }
}

impl Expr<Span> {
    // note that the spans generated are nonsense

    pub fn arbitrary() -> impl Strategy<Value = Self> {
        Self::gen(Rc::new(Default::default()))
    }

    pub fn gen(config: Rc<ExprGenConfig>) -> impl Strategy<Value = Self> {
        let start_depth = config.depth.clone();
        Self::gen_nested(config, start_depth, HashSet::new())
    }

    fn gen_nested(
        config: Rc<ExprGenConfig>,
        depth: std::ops::Range<usize>,
        bound_identifiers: HashSet<Identifier>,
    ) -> impl Strategy<Value = Self> {
        let mut choices: Vec<BoxedStrategy<Self>> = Vec::new();

        if depth.start == 0 {
            choices.push(
                Primitive::arbitrary()
                    .prop_map(|value| Expr::Primitive {
                        annotation: 0.into(),
                        value,
                    })
                    .boxed(),
            );

            if !bound_identifiers.is_empty() {
                let bound = bound_identifiers.clone();
                choices.push(
                    proptest::arbitrary::any::<proptest::sample::Index>()
                        .prop_map(move |index| Expr::Identifier {
                            annotation: 0.into(),
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
                let conf = config.clone();
                let bound = bound_identifiers.clone();
                proptest::arbitrary::any::<Operation>()
                    .prop_flat_map(move |operation| {
                        (
                            Self::gen_nested(conf.clone(), next_start..next_end, bound.clone()),
                            Self::gen_nested(conf.clone(), next_start..next_end, bound.clone()),
                        )
                            .prop_map(move |(left, right)| Expr::Infix {
                                annotation: 0.into(),
                                operation,
                                left: left.into(),
                                right: right.into(),
                            })
                    })
                    .boxed()
            });

            choices.push({
                let conf = config;
                let bound = bound_identifiers;
                conf.gen_identifier
                    .clone()
                    .prop_flat_map(move |name| {
                        let gen_value =
                            Self::gen_nested(conf.clone(), next_start..next_end, bound.clone());
                        let gen_inner = Self::gen_nested(
                            conf.clone(),
                            next_start..next_end,
                            bound.update(name.clone()),
                        );
                        (gen_value, gen_inner).prop_map(move |(value, inner)| Expr::Let {
                            annotation: 0.into(),
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
