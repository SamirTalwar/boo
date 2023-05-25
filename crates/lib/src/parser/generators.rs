// note that the spans generated are nonsense

use std::rc::Rc;

use im::HashSet;
use proptest::prelude::*;

use crate::identifier::Identifier;

use super::*;

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

pub fn arbitrary() -> impl Strategy<Value = Expr> {
    gen(Rc::new(Default::default()))
}

pub fn gen(config: Rc<ExprGenConfig>) -> impl Strategy<Value = Expr> {
    let start_depth = config.depth.clone();
    gen_nested(config, start_depth, HashSet::new())
}

fn gen_nested(
    config: Rc<ExprGenConfig>,
    depth: std::ops::Range<usize>,
    bound_identifiers: HashSet<Identifier>,
) -> impl Strategy<Value = Expr> {
    let mut choices: Vec<BoxedStrategy<Expr>> = Vec::new();

    if depth.start == 0 {
        choices.push(
            Primitive::arbitrary()
                .prop_map(|value| {
                    Spanned {
                        span: 0.into(),
                        value: Expression::Primitive { value },
                    }
                    .into()
                })
                .boxed(),
        );

        if !bound_identifiers.is_empty() {
            let bound = bound_identifiers.clone();
            choices.push(
                proptest::arbitrary::any::<proptest::sample::Index>()
                    .prop_map(move |index| {
                        Spanned {
                            span: 0.into(),
                            value: Expression::Identifier {
                                name: bound.iter().nth(index.index(bound.len())).unwrap().clone(),
                            },
                        }
                        .into()
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
                        gen_nested(conf.clone(), next_start..next_end, bound.clone()),
                        gen_nested(conf.clone(), next_start..next_end, bound.clone()),
                    )
                        .prop_map(move |(left, right)| {
                            {
                                Spanned {
                                    span: 0.into(),
                                    value: Expression::Infix {
                                        operation,
                                        left,
                                        right,
                                    },
                                }
                            }
                            .into()
                        })
                })
                .boxed()
        });

        choices.push({
            let conf = config.clone();
            let bound = bound_identifiers.clone();
            gen_unused_identifier(config, bound_identifiers)
                .prop_flat_map(move |name| {
                    let gen_value = gen_nested(conf.clone(), next_start..next_end, bound.clone());
                    let gen_inner = gen_nested(
                        conf.clone(),
                        next_start..next_end,
                        bound.update(name.clone()),
                    );
                    (gen_value, gen_inner).prop_map(move |(value, inner)| {
                        Spanned {
                            span: 0.into(),
                            value: Expression::Assign {
                                name: name.clone(),
                                value,
                                inner,
                            },
                        }
                        .into()
                    })
                })
                .boxed()
        });
    }

    proptest::strategy::Union::new(choices)
}

fn gen_unused_identifier(
    config: Rc<ExprGenConfig>,
    bound_identifiers: HashSet<Identifier>,
) -> impl Strategy<Value = Identifier> {
    let conf = config.clone();
    config.gen_identifier.clone().prop_flat_map(move |name| {
        if bound_identifiers.contains(&name) {
            gen_unused_identifier(conf.clone(), bound_identifiers.clone()).boxed()
        } else {
            Just(name).boxed()
        }
    })
}
