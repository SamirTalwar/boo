//! generators for asts. used for testing and program synthesis.

use std::fmt::Debug;
use std::rc::Rc;

use im::HashMap;
use proptest::prelude::*;

use boo_core::identifier::Identifier;
use boo_core::primitive::Primitive;
use boo_core::types::{KnownType, Type};
use boo_language::*;

type ExprStrategyValue = (Expr, Rc<KnownType>);
type ExprStrategy = BoxedStrategy<ExprStrategyValue>;

type Bindings = HashMap<Identifier, Rc<KnownType>>;

/// The generator configuration.
#[derive(Debug)]
pub struct ExprGenConfig {
    /// The minimum and maximum depth of each branch of the expression.
    ///
    /// Note that the maximum depth may be violated sometimes; consider it
    /// best-effort.
    pub depth: std::ops::Range<usize>,
    /// The specific strategy for generating identifiers.
    pub gen_identifier: Rc<BoxedStrategy<Identifier>>,
}

impl Default for ExprGenConfig {
    fn default() -> Self {
        Self {
            depth: 0..4,
            gen_identifier: Rc::new(Identifier::arbitrary().boxed()),
        }
    }
}

/// A strategy for generating expressions, using the default [`ExprGenConfig`].
pub fn arbitrary() -> impl Strategy<Value = Expr> {
    gen(Rc::new(Default::default()))
}

/// Creates a strategy for generating expresions according to the configuration.
pub fn gen(config: Rc<ExprGenConfig>) -> impl Strategy<Value = Expr> {
    Just(KnownType::Integer)
        .prop_flat_map(move |target_type| {
            let start_depth = config.depth.clone();
            gen_nested(
                config.clone(),
                start_depth,
                Type::Known(target_type.into()),
                HashMap::new(),
            )
        })
        .prop_map(|(expr, _)| expr)
}

/// Generates an expression of the target type (or any type, if it's not
/// specified).
fn gen_nested(
    config: Rc<ExprGenConfig>,
    depth: std::ops::Range<usize>,
    target_type: Type,
    bindings: Bindings,
) -> ExprStrategy {
    let mut choices: Vec<(u32, ExprStrategy)> = Vec::new();
    let next_depth = {
        let next_start = if depth.start == 0 { 0 } else { depth.start - 1 };
        let next_end = if depth.end == 0 { 0 } else { depth.end - 1 };
        next_start..next_end
    };

    // if we are allowed to generate a leaf:
    if depth.start == 0 {
        // generate primitives
        if let Some(strategy) = gen_primitive(target_type.clone()) {
            choices.push((1, strategy));
        }

        // generate references to already-bound variables (in `bindings`)
        if let Some(strategy) = gen_variable_reference(target_type.clone(), bindings.clone()) {
            choices.push((10, strategy));
        }
    }

    // if this node can have children:
    if depth.end > 0 {
        // generate variable assignments
        choices.push((
            2,
            gen_assignment(
                config.clone(),
                next_depth.clone(),
                target_type.clone(),
                bindings.clone(),
            ),
        ));

        // generate functions
        if let Some(strategy) = gen_function(
            config.clone(),
            next_depth.clone(),
            target_type.clone(),
            bindings.clone(),
        ) {
            choices.push((2, strategy));
        }
    }

    // If we continuously generate nodes that do not introduce new bindings,
    // we can end up with uncontrollable recursion. By limiting these types of
    // nodes to depth (max_depth - 2) or higher, we try to avoid this (most of
    // the time).
    if depth.end > 1 {
        // generate function application
        choices.push((
            2,
            gen_apply(
                config.clone(),
                next_depth.clone(),
                target_type.clone(),
                bindings.clone(),
            ),
        ));

        // generate infix computations
        if let Some(strategy) = gen_infix(
            config.clone(),
            next_depth,
            target_type.clone(),
            bindings.clone(),
        ) {
            choices.push((2, strategy));
        }
    }

    if choices.is_empty() {
        // increase the depth and try again
        gen_nested(config, depth.start..(depth.end + 1), target_type, bindings)
    } else {
        prop::strategy::Union::new_weighted(choices).boxed()
    }
}

/// Generates an identifier that has not already been bound.
fn gen_unused_identifier(
    config: Rc<ExprGenConfig>,
    bindings: Bindings,
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

/// Generates a primitive of the given type.
/// Returns `None` if there are no primitives of the target type.
fn gen_primitive(target_type: Type) -> Option<ExprStrategy> {
    Primitive::arbitrary_of_type(target_type).map(|s| s.prop_map(make_primitive_expr).boxed())
}

fn make_primitive_expr(value: Primitive) -> ExprStrategyValue {
    let value_type = value.get_type();
    let expr = Expr::new(0.into(), Expression::Primitive(value));
    (expr, value_type.into())
}

/// Generates a reference to a variable of the given type.
/// Returns `None` if there are no variables of the target type.
fn gen_variable_reference(target_type: Type, bindings: Bindings) -> Option<ExprStrategy> {
    let bindings_of_target_type = match target_type {
        Type::Unknown => bindings,
        Type::Known(expected) => bindings
            .iter()
            .filter(|(_, actual)| **actual == expected)
            .map(|(expr, expr_type)| (expr.clone(), expr_type.clone()))
            .collect(),
    };
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
                    let expr = Expr::new(0.into(), Expression::Identifier(name.clone()));
                    (expr, typ.clone())
                })
                .boxed(),
        )
    }
}

/// Generates an assignment.
fn gen_assignment(
    config: Rc<ExprGenConfig>,
    next_depth: std::ops::Range<usize>,
    target_type: Type,
    bindings: Bindings,
) -> ExprStrategy {
    gen_unused_identifier(config.clone(), bindings.clone())
        .prop_flat_map(move |name| {
            let config_ = config.clone();
            let next_depth_ = next_depth.clone();
            let target_type_ = target_type.clone();
            let bindings_ = bindings.clone();
            gen_nested(
                config_.clone(),
                next_depth.clone(),
                Type::Unknown,
                bindings_.clone(),
            )
            .prop_flat_map(move |(value, value_type): ExprStrategyValue| {
                let name_ = name.clone();
                let value_ = value;
                gen_nested(
                    config_.clone(),
                    next_depth_.clone(),
                    target_type_.clone(),
                    bindings_.update(name.clone(), value_type),
                )
                .prop_map(move |(inner, inner_type)| {
                    let expr = Expr::new(
                        0.into(),
                        Expression::Assign(Assign {
                            name: name_.clone(),
                            value: value_.clone(),
                            inner,
                        }),
                    );
                    (expr, inner_type)
                })
            })
        })
        .boxed()
}

/// Generates a function of the given type.
/// If the target type is not a function type, returns `None`.
fn gen_function(
    config: Rc<ExprGenConfig>,
    next_depth: std::ops::Range<usize>,
    target_type: Type,
    bindings: Bindings,
) -> Option<ExprStrategy> {
    match target_type {
        // cannot generate functions for parameters of unknown type without some kind of unification
        Type::Known(known) => {
            if let KnownType::Function {
                parameter: Type::Known(parameter_type),
                body: target_body_type,
            } = (*known).clone()
            {
                Some(
                    gen_unused_identifier(config.clone(), bindings.clone())
                        .prop_flat_map(move |parameter| {
                            let parameter_ = parameter.clone();
                            let parameter_type_ = parameter_type.clone();
                            gen_nested(
                                config.clone(),
                                next_depth.clone(),
                                target_body_type.clone(),
                                bindings.update(parameter, parameter_type_.clone()),
                            )
                            .prop_map(move |(body, body_type)| {
                                let expr = Expr::new(
                                    0.into(),
                                    Expression::Function(Function {
                                        parameter: parameter_.clone(),
                                        body,
                                    }),
                                );
                                let expr_type = KnownType::Function {
                                    parameter: Type::Known(parameter_type_.clone()),
                                    body: Type::Known(body_type),
                                }
                                .into();
                                (expr, expr_type)
                            })
                        })
                        .boxed(),
                )
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Generates a function application.
fn gen_apply(
    config: Rc<ExprGenConfig>,
    next_depth: std::ops::Range<usize>,
    target_type: Type,
    bindings: Bindings,
) -> ExprStrategy {
    gen_nested(
        config.clone(),
        next_depth.clone(),
        Type::Unknown,
        bindings.clone(),
    )
    .prop_flat_map(move |(argument, argument_type): ExprStrategyValue| {
        gen_nested(
            config.clone(),
            next_depth.clone(),
            Type::Known(
                KnownType::Function {
                    parameter: Type::Known(argument_type),
                    body: target_type.clone(),
                }
                .into(),
            ),
            bindings.clone(),
        )
        .prop_map(move |(function, function_type)| {
            let expr = Expr::new(
                0.into(),
                Expression::Apply(Apply {
                    function,
                    argument: argument.clone(),
                }),
            );
            let expr_type = match function_type.as_ref() {
                KnownType::Function {
                    body: Type::Known(body_type),
                    ..
                } => body_type.clone(),
                _ => panic!("No function return type provided."),
            };
            (expr, expr_type)
        })
    })
    .boxed()
}

/// Generates an infix operation of the given type.
/// If the type is not `Integer`, returns `None`.
fn gen_infix(
    config: Rc<ExprGenConfig>,
    next_depth: std::ops::Range<usize>,
    target_type: Type,
    bindings: Bindings,
) -> Option<ExprStrategy> {
    match target_type {
        Type::Known(known) if *known == KnownType::Integer => Some(
            proptest::arbitrary::any::<Operation>()
                .prop_flat_map(move |operation| {
                    (
                        gen_nested(
                            config.clone(),
                            next_depth.clone(),
                            Type::Known(KnownType::Integer.into()),
                            bindings.clone(),
                        ),
                        gen_nested(
                            config.clone(),
                            next_depth.clone(),
                            Type::Known(KnownType::Integer.into()),
                            bindings.clone(),
                        ),
                    )
                        .prop_map(move |((left, _), (right, _))| {
                            let expr = Expr::new(
                                0.into(),
                                Expression::Infix(Infix {
                                    operation,
                                    left,
                                    right,
                                }),
                            );
                            (expr, KnownType::Integer.into())
                        })
                })
                .boxed(),
        ),
        _ => None,
    }
}
