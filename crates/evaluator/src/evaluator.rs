//! Evaluates a [pooled `Expr`][super::pooler::ast::Expr].

use std::borrow::Cow;
use std::sync::Arc;

use im::HashMap;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::identifier::*;
use boo_core::operation::*;
use boo_core::primitive::*;

use crate::pooler::ast::*;
use crate::thunk::Thunk;

/// An evaluation result. This can be either a primitive value or a closure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evaluated {
    Primitive(Primitive),
    Function(Function<Expr>),
}

impl std::fmt::Display for Evaluated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluated::Primitive(x) => x.fmt(f),
            Evaluated::Function(x) => x.fmt(f),
        }
    }
}

/// An interim evaluation result, with the same lifetime as the pool being
/// evaluated.
#[derive(Debug, Clone)]
enum EvaluationProgress<'a> {
    Primitive(Cow<'a, Primitive>),
    Closure(&'a Function<Expr>, Bindings<'a>),
}

impl<'a> EvaluationProgress<'a> {
    /// Concludes evaluation.
    fn finish(self) -> Evaluated {
        match self {
            Self::Primitive(x) => Evaluated::Primitive(x.into_owned()),
            Self::Closure(x, _) => Evaluated::Function(x.clone()),
        }
    }
}

type UnevaluatedBinding<'a> = (Expr, Bindings<'a>);
type EvaluatedBinding<'a> = Result<EvaluationProgress<'a>>;

/// The set of bindings in a given scope.
///
/// The variables bound in a specific scope are a mapping from an identifier to
/// the underlying expression. This expression is evaluated lazily, but only
/// once, using [`Thunk`].
#[derive(Debug, Clone)]
struct Bindings<'a>(
    HashMap<Cow<'a, Identifier>, Thunk<UnevaluatedBinding<'a>, EvaluatedBinding<'a>>>,
);

impl<'a> Bindings<'a> {
    /// Constructs an empty set of bindings.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Resolves a given identifier by evaluating its binding.
    pub fn resolve(
        &mut self,
        identifier: &Identifier,
        resolver: impl Fn(Option<UnevaluatedBinding<'a>>) -> EvaluatedBinding<'a>,
    ) -> EvaluatedBinding<'a> {
        match self.0.get_mut(identifier) {
            Some(thunk) => {
                let result = thunk.resolve_by(move |(pool_ref, bindings)| {
                    resolver(Some((*pool_ref, bindings.clone())))
                });
                Arc::try_unwrap(result).unwrap_or_else(|arc| (*arc).clone())
            }
            None => resolver(None),
        }
    }

    /// Adds a new binding to the set.
    pub fn with(&self, identifier: &'a Identifier, expression: Expr) -> Self {
        Self(self.0.update(
            Cow::Borrowed(identifier),
            Thunk::unresolved((expression, self.clone())),
        ))
    }
}

/// Evaluate a [pooled `Expr`][super::pooler::ast::Expr].
pub fn evaluate(pool: &ExprPool) -> Result<Evaluated> {
    let evaluated = evaluate_(pool, Expr::from_root(pool), Bindings::new())?;
    Ok(evaluated.finish())
}

/// Evaluates an expression from a pool in a given scope, with the associated
/// bindings.
///
/// The bindings are modified by assignment, accessed when evaluating an
/// identifier, and captured by closures when a function is evaluated.
fn evaluate_<'a>(
    pool: &'a ExprPool,
    expr_ref: Expr,
    bindings: Bindings<'a>,
) -> Result<EvaluationProgress<'a>> {
    let expr = expr_ref.read_from(pool);
    match &expr.value {
        Expression::Primitive(value) => Ok(EvaluationProgress::Primitive(Cow::Borrowed(value))),
        Expression::Identifier(name) => bindings.clone().resolve(name, |thunk| match thunk {
            Some((value_ref, thunk_bindings)) => evaluate_(pool, value_ref, thunk_bindings),
            None => Err(Error::UnknownVariable {
                span: expr.span,
                name: name.to_string(),
            }),
        }),
        Expression::Assign(Assign {
            name,
            value: value_ref,
            inner: inner_ref,
        }) => evaluate_(pool, *inner_ref, bindings.with(name, *value_ref)),
        Expression::Function(function) => {
            Ok(EvaluationProgress::Closure(function, bindings.clone()))
        }
        Expression::Apply(Apply {
            function: function_ref,
            argument: argument_ref,
        }) => {
            let function_result = evaluate_(pool, *function_ref, bindings.clone())?;
            match function_result {
                EvaluationProgress::Closure(
                    Function {
                        parameter,
                        body: body_ref,
                    },
                    function_bindings,
                ) => evaluate_(
                    pool,
                    *body_ref,
                    function_bindings.with(parameter, *argument_ref),
                ),
                _ => Err(Error::InvalidFunctionApplication { span: expr.span }),
            }
        }
        Expression::Infix(Infix {
            operation,
            left: left_ref,
            right: right_ref,
        }) => {
            let left_result = evaluate_(pool, *left_ref, bindings.clone())?;
            let right_result = evaluate_(pool, *right_ref, bindings)?;
            Ok(evaluate_infix(*operation, left_result, right_result))
        }
    }
}

fn evaluate_infix<'a>(
    operation: Operation,
    left: EvaluationProgress<'a>,
    right: EvaluationProgress<'a>,
) -> EvaluationProgress<'a> {
    match (&left, &right) {
        (EvaluationProgress::Primitive(left), EvaluationProgress::Primitive(right)) => {
            match (left.as_ref(), right.as_ref()) {
                (Primitive::Integer(left), Primitive::Integer(right)) => {
                    EvaluationProgress::Primitive(Cow::Owned(match operation {
                        Operation::Add => Primitive::Integer(left + right),
                        Operation::Subtract => Primitive::Integer(left - right),
                        Operation::Multiply => Primitive::Integer(left * right),
                    }))
                }
            }
        }
        _ => panic!(
            "evaluate_infix branch is not implemented for:\n  left:   {:?}\nright:  {:?}",
            left, right
        ),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use boo_core::ast::builders;
    use boo_core::span::Spanned;
    use boo_test_helpers::proptest::*;

    use crate::pooler::pool::{leaky_pool_with, pool_with};

    use super::*;

    fn pool_of(expr: simple::Expr) -> ExprPool {
        pool_with(|pool| {
            expr.transform(&mut |_, expression| Expr::insert(pool, 0.into(), expression));
        })
    }

    #[test]
    fn test_evaluating_a_primitive() {
        check(&Primitive::arbitrary(), |value| {
            // input: `value`
            let input = pool_of(builders::primitive((), value.clone()));
            let expected = Evaluated::Primitive(value);

            let actual = evaluate(&input);

            prop_assert_eq!(actual, Ok(expected));
            Ok(())
        })
    }

    #[test]
    fn test_evaluating_assignment() {
        check(
            &(Identifier::arbitrary(), Primitive::arbitrary()),
            |(name, value)| {
                // input: let `name` = `value` in `name`
                let input = pool_of(builders::assign(
                    (),
                    name.clone(),
                    builders::primitive((), value.clone()),
                    builders::identifier((), name),
                ));
                let expected = Evaluated::Primitive(value);

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_evaluating_variable_use() {
        check(
            &(
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(name, variable, constant)| {
                let sum = &variable + &constant;
                // input: let `name` = `variable` in `constant` + `name`
                let input = pool_of(builders::assign(
                    (),
                    name.clone(),
                    builders::primitive_integer((), variable),
                    builders::infix(
                        (),
                        Operation::Add,
                        builders::identifier((), name),
                        builders::primitive_integer((), constant),
                    ),
                ));
                let expected = Evaluated::Primitive(Primitive::Integer(sum));

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_evaluating_an_unknown_variable() {
        check(&Identifier::arbitrary(), |name| {
            let input = pool_with(|pool| {
                pool.add(Spanned {
                    span: (5..10).into(),
                    value: Expression::Identifier(name.clone()),
                });
            });

            let actual = evaluate(&input);

            prop_assert_eq!(
                actual,
                Err(Error::UnknownVariable {
                    span: (5..10).into(),
                    name: name.to_string()
                })
            );
            Ok(())
        })
    }

    #[test]
    fn test_an_isolated_function() {
        check(&Identifier::arbitrary(), |parameter| {
            // input: fn `parameter` -> `parameter`
            let (input, body_ref) = leaky_pool_with(|pool| {
                let body_ref = crate::pooler::builders::identifier(pool, parameter.clone());
                crate::pooler::builders::function(pool, parameter.clone(), body_ref);
                body_ref
            });
            let expected = Evaluated::Function(Function {
                parameter,
                body: body_ref,
            });

            let actual = evaluate(&input);

            prop_assert_eq!(actual, Ok(expected));
            Ok(())
        })
    }

    #[test]
    fn test_simple_function_application() {
        check(
            &(Identifier::arbitrary(), Integer::arbitrary()),
            |(parameter, argument)| {
                // input: (fn `parameter` -> `parameter`) `argument`
                let input = pool_of(builders::apply(
                    (),
                    builders::function((), parameter.clone(), builders::identifier((), parameter)),
                    builders::primitive_integer((), argument.clone()),
                ));
                let expected = Evaluated::Primitive(Primitive::Integer(argument));

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_complex_function_application() {
        check(
            &(
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(parameter, multiplier, argument_left, argument_right)| {
                // input:
                //   (fn `parameter` -> `parameter` * `multiplier`)
                //     (`argument_left` + `argument_right`)
                let input = pool_of(builders::apply(
                    (),
                    builders::function(
                        (),
                        parameter.clone(),
                        builders::infix(
                            (),
                            Operation::Multiply,
                            builders::identifier((), parameter),
                            builders::primitive_integer((), multiplier.clone()),
                        ),
                    ),
                    builders::infix(
                        (),
                        Operation::Add,
                        builders::primitive_integer((), argument_left.clone()),
                        builders::primitive_integer((), argument_right.clone()),
                    ),
                ));
                let expected = Evaluated::Primitive(Primitive::Integer(
                    (argument_left + argument_right) * multiplier,
                ));

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_closing_a_function_over_a_variable() {
        check(
            &(
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Identifier::arbitrary(),
                Integer::arbitrary(),
            ),
            |(outer_variable_name, outer_variable_value, parameter, argument_value)| {
                prop_assume!(outer_variable_name != parameter);

                // input:
                //   (let `outer_variable_name` = `outer_variable_value`
                //        in fn `parameter` -> `outer_variable_name` + `parameter`)
                //     `argument_value
                let input = pool_of(builders::apply(
                    (),
                    builders::assign(
                        (),
                        outer_variable_name.clone(),
                        builders::primitive_integer((), outer_variable_value.clone()),
                        builders::function(
                            (),
                            parameter.clone(),
                            builders::infix(
                                (),
                                Operation::Add,
                                builders::identifier((), outer_variable_name),
                                builders::identifier((), parameter),
                            ),
                        ),
                    ),
                    builders::primitive_integer((), argument_value.clone()),
                ));
                let expected =
                    Evaluated::Primitive(Primitive::Integer(outer_variable_value + argument_value));

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_does_not_close_functions_over_variables_out_of_scope() {
        check(
            &(
                Identifier::arbitrary(),
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Identifier::arbitrary(),
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(
                function_name,
                outer_variable_name,
                outer_variable_value,
                parameter_name,
                external_variable_name,
                external_variable_value,
                argument_value,
            )| {
                prop_assume!(
                    std::collections::HashSet::from([
                        function_name.clone(),
                        outer_variable_name.clone(),
                        parameter_name.clone(),
                        external_variable_name.clone(),
                    ])
                    .len()
                        == 4
                );

                // let `function_name` =
                //   (let `outer_variable_name` = `outer_variable_value`
                //        in fn `parameter_name` -> `outer_variable_name` + `parameter_name` + `external_variable_name`)
                //     in let `external_variable_name` = `external_variable_value`
                //            in `function_name` `argument_value`
                let input = pool_of(builders::assign(
                    (),
                    function_name.clone(),
                    builders::assign(
                        (),
                        outer_variable_name.clone(),
                        builders::primitive_integer((), outer_variable_value),
                        builders::function(
                            (),
                            parameter_name.clone(),
                            builders::infix(
                                (),
                                Operation::Add,
                                builders::identifier((), outer_variable_name),
                                builders::infix(
                                    (),
                                    Operation::Add,
                                    builders::identifier((), parameter_name),
                                    builders::identifier((), external_variable_name.clone()),
                                ),
                            ),
                        ),
                    ),
                    builders::assign(
                        (),
                        external_variable_name.clone(),
                        builders::primitive_integer((), external_variable_value),
                        builders::apply(
                            (),
                            builders::identifier((), function_name),
                            builders::primitive_integer((), argument_value),
                        ),
                    ),
                ));

                let actual = evaluate(&input);

                prop_assert_eq!(
                    actual,
                    Err(Error::UnknownVariable {
                        span: (0..0).into(),
                        name: external_variable_name.to_string()
                    })
                );
                Ok(())
            },
        )
    }

    #[test]
    fn test_evaluating_addition() {
        test_evaluating_an_operation(Operation::Add, |x, y| x + y)
    }

    #[test]
    fn test_evaluating_subtraction() {
        test_evaluating_an_operation(Operation::Subtract, |x, y| x - y)
    }

    #[test]
    fn test_evaluating_multiplication() {
        test_evaluating_an_operation(Operation::Multiply, |x, y| x * y)
    }

    fn test_evaluating_an_operation(
        operation: Operation,
        implementation: impl Fn(&Integer, &Integer) -> Integer,
    ) {
        check(
            &(Integer::arbitrary(), Integer::arbitrary()),
            |(left, right)| {
                let expected =
                    Evaluated::Primitive(Primitive::Integer(implementation(&left, &right)));
                // input: `left` `operation` `right`
                let input = pool_of(builders::infix(
                    (),
                    operation,
                    builders::primitive_integer((), left),
                    builders::primitive_integer((), right),
                ));

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }
}
