//! Generators for ASTs. Used for testing and program synthesis.

use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;

use im::HashMap;
use proptest::prelude::*;

use boo_core::identifier::Identifier;
use boo_core::primitive::Primitive;
use boo_core::types::{Type, TypeRef};
use boo_language::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetType {
    Unknown,
    Known(Arc<Type<Self>>),
}

impl From<Type<Self>> for TargetType {
    fn from(value: Type<Self>) -> Self {
        Self::Known(Arc::new(value))
    }
}

impl TypeRef for TargetType {}

type ExprStrategyValue = (Expr, TargetType);
type ExprStrategy = BoxedStrategy<ExprStrategyValue>;

type Bindings = HashMap<Identifier, TargetType>;

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
    Just(Type::<TargetType>::Integer.into())
        .prop_flat_map(move |target_type| {
            let start_depth = config.depth.clone();
            gen_nested(config.clone(), start_depth, target_type, HashMap::new())
        })
        .prop_map(|(expr, _)| expr)
}

/// Generates an expression of the target type (or any type, if it's not
/// specified).
fn gen_nested(
    config: Rc<ExprGenConfig>,
    depth: std::ops::Range<usize>,
    target_type: TargetType,
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
            choices.push((1, strategy.prop_map(make_primitive_expr).boxed()));
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
        // generate pattern matches
        choices.push((
            2,
            gen_match(
                config.clone(),
                next_depth.clone(),
                target_type.clone(),
                bindings.clone(),
            ),
        ));

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
fn gen_primitive(target_type: TargetType) -> Option<BoxedStrategy<Primitive>> {
    match target_type {
        TargetType::Unknown => Some(Primitive::arbitrary().boxed()),
        TargetType::Known(t) => Primitive::arbitrary_of_type(t.as_ref()),
    }
}

fn make_primitive_expr(value: Primitive) -> ExprStrategyValue {
    let value_type = value.get_type();
    let expr = Expr::new(0.into(), Expression::Primitive(value));
    (expr, value_type)
}

/// Generates a reference to a variable of the given type.
/// Returns `None` if there are no variables of the target type.
fn gen_variable_reference(target_type: TargetType, bindings: Bindings) -> Option<ExprStrategy> {
    let bindings_of_target_type = match target_type {
        TargetType::Unknown => bindings,
        TargetType::Known(_) => bindings
            .iter()
            .filter(|(_, actual)| **actual == target_type)
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
    target_type: TargetType,
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
                TargetType::Unknown,
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
    target_type: TargetType,
    bindings: Bindings,
) -> Option<ExprStrategy> {
    match target_type {
        // cannot generate functions for parameters of unknown type without some kind of unification
        TargetType::Known(known) => match known.as_ref() {
            Type::Function {
                parameter: ref parameter_type @ TargetType::Known(_),
                body: ref target_body_type,
            } => {
                let parameter_type_ = parameter_type.clone();
                let target_body_type_ = target_body_type.clone();
                Some(
                    gen_unused_identifier(config.clone(), bindings.clone())
                        .prop_flat_map(move |parameter| {
                            let parameter_ = parameter.clone();
                            let parameter_type__ = parameter_type_.clone();
                            gen_nested(
                                config.clone(),
                                next_depth.clone(),
                                target_body_type_.clone(),
                                bindings.update(parameter, parameter_type_.clone()),
                            )
                            .prop_map(move |(body, body_type)| {
                                let expr = Expr::new(
                                    0.into(),
                                    Expression::Function(Function {
                                        parameters: vec![parameter_.clone()],
                                        body,
                                    }),
                                );
                                let expr_type = Type::Function {
                                    parameter: parameter_type__.clone(),
                                    body: body_type,
                                }
                                .into();
                                (expr, expr_type)
                            })
                        })
                        .boxed(),
                )
            }
            _ => None,
        },
        _ => None,
    }
}

/// Generates a pattern match.
///
/// It always has a default case.
fn gen_match(
    config: Rc<ExprGenConfig>,
    next_depth: std::ops::Range<usize>,
    target_type: TargetType,
    bindings: Bindings,
) -> ExprStrategy {
    gen_nested(
        config.clone(),
        next_depth.clone(),
        TargetType::Unknown,
        bindings.clone(),
    )
    .prop_flat_map(move |(value, value_type): ExprStrategyValue| {
        let config_ = config.clone();
        let next_depth_ = next_depth.clone();
        let target_type_ = target_type.clone();
        let bindings_ = bindings.clone();
        proptest::collection::vec(
            gen_pattern(
                config.clone(),
                next_depth.clone(),
                value_type,
                target_type.clone(),
                bindings.clone(),
            ),
            0..5,
        )
        .prop_flat_map(move |patterns| {
            gen_nested(
                config_.clone(),
                next_depth_.clone(),
                target_type_.clone(),
                bindings_.clone(),
            )
            .prop_map(move |(anything_result, anything_type)| {
                let mut patterns_with_base_case = patterns.clone();
                patterns_with_base_case.push((Pattern::Anything, anything_result, anything_type));
                patterns_with_base_case
            })
        })
        .prop_map(move |patterns| {
            let expr = Expr::new(
                0.into(),
                Expression::Match(Match {
                    value: value.clone(),
                    patterns: patterns
                        .iter()
                        .map(|(pattern, result, _)| PatternMatch {
                            pattern: pattern.clone(),
                            result: result.clone(),
                        })
                        .collect(),
                }),
            );
            let expr_type = patterns.first().unwrap().2.clone();
            (expr, expr_type)
        })
    })
    .boxed()
}

/// Generates a single pattern.
fn gen_pattern(
    config: Rc<ExprGenConfig>,
    next_depth: std::ops::Range<usize>,
    pattern_type: TargetType,
    target_type: TargetType,
    bindings: Bindings,
) -> impl Strategy<Value = (Pattern, Expr, TargetType)> {
    let mut choices: Vec<BoxedStrategy<Pattern>> = vec![];
    if let Some(primitive_strategy) =
        gen_primitive(pattern_type).map(|strategy| strategy.prop_map(Pattern::Primitive))
    {
        choices.push(primitive_strategy.boxed());
    };
    choices.push(Just(Pattern::Anything).boxed());
    prop::strategy::Union::new(choices).prop_flat_map(move |pattern| {
        gen_nested(
            config.clone(),
            next_depth.clone(),
            target_type.clone(),
            bindings.clone(),
        )
        .prop_map(move |(expr, expr_type)| (pattern.clone(), expr, expr_type))
    })
}

/// Generates a function application.
fn gen_apply(
    config: Rc<ExprGenConfig>,
    next_depth: std::ops::Range<usize>,
    target_type: TargetType,
    bindings: Bindings,
) -> ExprStrategy {
    gen_nested(
        config.clone(),
        next_depth.clone(),
        TargetType::Unknown,
        bindings.clone(),
    )
    .prop_flat_map(move |(argument, argument_type): ExprStrategyValue| {
        gen_nested(
            config.clone(),
            next_depth.clone(),
            TargetType::Known(
                Type::Function {
                    parameter: argument_type,
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
            let expr_type = match function_type {
                TargetType::Known(known) => match known.as_ref() {
                    Type::Function {
                        body: body_type @ TargetType::Known(_),
                        ..
                    } => body_type.clone(),
                    _ => panic!("No function return type provided."),
                },
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
    target_type: TargetType,
    bindings: Bindings,
) -> Option<ExprStrategy> {
    match target_type {
        TargetType::Known(known) if *known == Type::Integer => Some(
            proptest::arbitrary::any::<Operation>()
                .prop_flat_map(move |operation| {
                    (
                        gen_nested(
                            config.clone(),
                            next_depth.clone(),
                            TargetType::Known(Type::Integer.into()),
                            bindings.clone(),
                        ),
                        gen_nested(
                            config.clone(),
                            next_depth.clone(),
                            TargetType::Known(Type::Integer.into()),
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
                            (expr, Type::Integer.into())
                        })
                })
                .boxed(),
        ),
        _ => None,
    }
}
