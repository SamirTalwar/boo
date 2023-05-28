// note that the spans generated are nonsense

use std::rc::Rc;

use im::HashMap;
use proptest::prelude::*;

use crate::identifier::Identifier;
use crate::types::Type;

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
    Just(Type::Integer)
        .prop_flat_map(move |target_type| {
            let start_depth = config.depth.clone();
            gen_nested(
                config.clone(),
                start_depth,
                Some(target_type),
                HashMap::new(),
            )
        })
        .prop_map(|(expr, _)| expr)
}

type ExprStrategy = BoxedStrategy<(Expr, Type)>;

fn gen_nested(
    config: Rc<ExprGenConfig>,
    depth: std::ops::Range<usize>,
    target_type: Option<Type>,
    bindings: HashMap<Identifier, Type>,
) -> impl Strategy<Value = (Expr, Type)> {
    let mut choices: Vec<ExprStrategy> = Vec::new();

    if depth.start == 0 {
        if let Some(strategy) = gen_primitive(target_type.clone()) {
            choices.push(strategy);
        }

        if let Some(strategy) = gen_variable_reference(target_type.clone(), bindings.clone()) {
            choices.push(strategy);
        }
    }

    if depth.end > 0 {
        let next_depth = (if depth.start == 0 { 0 } else { depth.start - 1 })..(depth.end - 1);

        choices.push(gen_assignment(
            config.clone(),
            next_depth.clone(),
            target_type.clone(),
            bindings.clone(),
        ));

        if let Some(strategy) = gen_infix(config, next_depth, target_type, bindings) {
            choices.push(strategy);
        }
    }

    proptest::strategy::Union::new(choices)
}

fn gen_unused_identifier(
    config: Rc<ExprGenConfig>,
    bindings: HashMap<Identifier, Type>,
) -> impl Strategy<Value = Identifier> {
    let conf = config.clone();
    config.gen_identifier.clone().prop_flat_map(move |name| {
        if bindings.contains_key(&name) {
            gen_unused_identifier(conf.clone(), bindings.clone()).boxed()
        } else {
            Just(name).boxed()
        }
    })
}

fn gen_primitive(target_type: Option<Type>) -> Option<ExprStrategy> {
    match target_type {
        None => Some(Primitive::arbitrary().prop_map(make_primitive_expr).boxed()),
        Some(t) => Primitive::arbitrary_of_type(t).map(|s| s.prop_map(make_primitive_expr).boxed()),
    }
}

fn make_primitive_expr(value: Primitive) -> (Expr, Type) {
    let value_type = value.get_type();
    let expr = Spanned {
        span: 0.into(),
        value: Expression::Primitive(value),
    }
    .into();
    (expr, value_type)
}

fn gen_variable_reference(
    target_type: Option<Type>,
    bindings: HashMap<Identifier, Type>,
) -> Option<ExprStrategy> {
    let bindings_of_target_type = target_type.map_or(bindings.clone(), |expected| {
        bindings
            .clone()
            .iter()
            .filter(|(_, actual)| **actual == expected)
            .map(|(expr, expr_type)| (expr.clone(), expr_type.clone()))
            .collect()
    });
    if bindings_of_target_type.is_empty() {
        None
    } else {
        Some(
            proptest::arbitrary::any::<proptest::sample::Index>()
                .prop_map(move |index| {
                    let (name, typ) = bindings_of_target_type
                        .iter()
                        .nth(index.index(bindings_of_target_type.len()))
                        .unwrap();
                    let expr = Spanned {
                        span: 0.into(),
                        value: Expression::Identifier(name.clone()),
                    }
                    .into();
                    (expr, typ.clone())
                })
                .boxed(),
        )
    }
}

fn gen_assignment(
    config: Rc<ExprGenConfig>,
    next_depth: std::ops::Range<usize>,
    target_type: Option<Type>,
    bindings: HashMap<Identifier, Type>,
) -> ExprStrategy {
    gen_unused_identifier(config.clone(), bindings.clone())
        .prop_flat_map(move |name| {
            let config_ = config.clone();
            let next_depth_ = next_depth.clone();
            let target_type_ = target_type.clone();
            let bindings_ = bindings.clone();
            gen_nested(config_.clone(), next_depth.clone(), None, bindings_.clone()).prop_flat_map(
                move |(value, value_type)| {
                    let name_ = name.clone();
                    let value_ = value;
                    gen_nested(
                        config_.clone(),
                        next_depth_.clone(),
                        target_type_.clone(),
                        bindings_.update(name.clone(), value_type),
                    )
                    .prop_map(move |(inner, inner_type)| {
                        let expr = Spanned {
                            span: 0.into(),
                            value: Expression::Assign(Assign {
                                name: name_.clone(),
                                value: value_.clone(),
                                inner,
                            }),
                        }
                        .into();
                        (expr, inner_type)
                    })
                },
            )
        })
        .boxed()
}

fn gen_infix(
    config: Rc<ExprGenConfig>,
    next_depth: std::ops::Range<usize>,
    target_type: Option<Type>,
    bindings: HashMap<Identifier, Type>,
) -> Option<ExprStrategy> {
    target_type.filter(|t| *t == Type::Integer).map(|_| {
        proptest::arbitrary::any::<Operation>()
            .prop_flat_map(move |operation| {
                (
                    gen_nested(
                        config.clone(),
                        next_depth.clone(),
                        Some(Type::Integer),
                        bindings.clone(),
                    ),
                    gen_nested(
                        config.clone(),
                        next_depth.clone(),
                        Some(Type::Integer),
                        bindings.clone(),
                    ),
                )
                    .prop_map(move |((left, _), (right, _))| {
                        let expr = Spanned {
                            span: 0.into(),
                            value: Expression::Infix(Infix {
                                operation,
                                left,
                                right,
                            }),
                        }
                        .into();
                        (expr, Type::Integer)
                    })
            })
            .boxed()
    })
}
