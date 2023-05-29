use std::borrow::Cow;
use std::sync::Arc;

use im::HashMap;

use crate::error::*;
use crate::identifier::*;
use crate::operation::*;
use crate::pooler::ast::*;
use crate::primitive::*;
use crate::thunk::Thunk;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evaluated {
    Primitive(Primitive),
    Function(Function),
}

impl std::fmt::Display for Evaluated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluated::Primitive(x) => x.fmt(f),
            Evaluated::Function(x) => x.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
enum EvaluationProgress<'a> {
    Primitive(Cow<'a, Primitive>),
    Function(&'a Function, Bindings<'a>),
}

impl<'a> EvaluationProgress<'a> {
    fn finish(self) -> Evaluated {
        match self {
            Self::Primitive(x) => Evaluated::Primitive(x.into_owned()),
            Self::Function(x, _) => Evaluated::Function(x.clone()),
        }
    }
}

type UnevaluatedBinding<'a> = (Expr, Bindings<'a>);
type EvaluatedBinding<'a> = Result<EvaluationProgress<'a>>;

#[derive(Debug, Clone)]
struct Bindings<'a>(
    HashMap<Cow<'a, Identifier>, Thunk<UnevaluatedBinding<'a>, EvaluatedBinding<'a>>>,
);

impl<'a> Bindings<'a> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

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

    pub fn with(&self, identifier: &'a Identifier, expression: Expr) -> Self {
        Self(self.0.update(
            Cow::Borrowed(identifier),
            Thunk::unresolved((expression, self.clone())),
        ))
    }
}

pub fn evaluate(pool: &ExprPool) -> Result<Evaluated> {
    let evaluated = evaluate_(pool, pool.root(), Bindings::new())?;
    Ok(evaluated.finish())
}

fn evaluate_<'a>(
    pool: &'a ExprPool,
    expr_ref: Expr,
    bindings: Bindings<'a>,
) -> Result<EvaluationProgress<'a>> {
    let expr = pool.get(expr_ref);
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
            Ok(EvaluationProgress::Function(function, bindings.clone()))
        }
        Expression::Apply(Apply {
            function: function_ref,
            argument: argument_ref,
        }) => {
            let function_result = evaluate_(pool, *function_ref, bindings.clone())?;
            match function_result {
                EvaluationProgress::Function(
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

    use boo_test_helpers::proptest::*;

    use crate::pooler::builders;
    use crate::pooler::pool::{leaky_pool_with, pool_with};
    use crate::span::Spanned;

    use super::*;

    #[test]
    fn test_evaluating_a_primitive() {
        check(&Primitive::arbitrary(), |value| {
            let input = pool_with(|pool| {
                builders::primitive(pool, value.clone());
            });
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
                let input = pool_with(|pool| {
                    let value_ref = builders::primitive(pool, value.clone());
                    let inner_ref = builders::identifier(pool, name.clone());
                    builders::assign(pool, name.clone(), value_ref, inner_ref);
                });
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
                let input = pool_with(|pool| {
                    let left_ref = builders::identifier(pool, name.clone());
                    let right_ref = builders::primitive_integer(pool, constant);
                    let value_ref = builders::primitive_integer(pool, variable);
                    let inner_ref = builders::infix(pool, Operation::Add, left_ref, right_ref);
                    builders::assign(pool, name.clone(), value_ref, inner_ref);
                });
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
            let (input, body_ref) = leaky_pool_with(|pool| {
                let body_ref = builders::identifier(pool, parameter.clone());
                builders::function(pool, parameter.clone(), body_ref);
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
                let input = pool_with(|pool| {
                    let body_ref = builders::identifier(pool, parameter.clone());
                    let function_ref = builders::function(pool, parameter.clone(), body_ref);
                    let argument_ref = builders::primitive_integer(pool, argument.clone());
                    builders::apply(pool, function_ref, argument_ref);
                });
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
                let input = pool_with(|pool| {
                    let body_left_ref = builders::identifier(pool, parameter.clone());
                    let body_right_ref = builders::primitive_integer(pool, multiplier.clone());
                    let body_ref =
                        builders::infix(pool, Operation::Multiply, body_left_ref, body_right_ref);
                    let function_ref = builders::function(pool, parameter.clone(), body_ref);
                    let argument_left_ref =
                        builders::primitive_integer(pool, argument_left.clone());
                    let argument_right_ref =
                        builders::primitive_integer(pool, argument_right.clone());
                    let argument_ref = builders::infix(
                        pool,
                        Operation::Add,
                        argument_left_ref,
                        argument_right_ref,
                    );
                    builders::apply(pool, function_ref, argument_ref);
                });
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
                let input = pool_with(|pool| {
                    let outer_variable_ref =
                        builders::primitive_integer(pool, outer_variable_value.clone());
                    let body_left_ref = builders::identifier(pool, outer_variable_name.clone());
                    let body_right_ref = builders::identifier(pool, parameter.clone());
                    let body_ref =
                        builders::infix(pool, Operation::Add, body_left_ref, body_right_ref);
                    let function_ref = builders::function(pool, parameter.clone(), body_ref);
                    let assignment_ref = builders::assign(
                        pool,
                        outer_variable_name,
                        outer_variable_ref,
                        function_ref,
                    );
                    let argument_ref = builders::primitive_integer(pool, argument_value.clone());
                    builders::apply(pool, assignment_ref, argument_ref);
                });
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
                let input = pool_with(|pool| {
                    // let f = (let x = 1 in fn y -> x + y + z) in let z = 3 in f 2
                    let outer_variable_ref =
                        builders::primitive_integer(pool, outer_variable_value.clone());
                    let function_body_x_ref =
                        builders::identifier(pool, outer_variable_name.clone());
                    let function_body_y_ref = builders::identifier(pool, parameter_name.clone());
                    let function_body_z_ref =
                        builders::identifier(pool, external_variable_name.clone());
                    let function_body_yz_ref = builders::infix(
                        pool,
                        Operation::Add,
                        function_body_y_ref,
                        function_body_z_ref,
                    );
                    let function_body_ref = builders::infix(
                        pool,
                        Operation::Add,
                        function_body_x_ref,
                        function_body_yz_ref,
                    );
                    let function_inner_declaration_ref =
                        builders::function(pool, parameter_name, function_body_ref);
                    let function_outer_declaration_ref = builders::assign(
                        pool,
                        outer_variable_name,
                        outer_variable_ref,
                        function_inner_declaration_ref,
                    );
                    let external_variable_ref =
                        builders::primitive_integer(pool, external_variable_value);
                    let function_ref = builders::identifier(pool, function_name.clone());
                    let argument_value_ref =
                        builders::primitive_integer(pool, argument_value.clone());
                    let apply_ref = builders::apply(pool, function_ref, argument_value_ref);
                    let external_assignment_ref = builders::assign(
                        pool,
                        external_variable_name.clone(),
                        external_variable_ref,
                        apply_ref,
                    );
                    builders::assign(
                        pool,
                        function_name,
                        function_outer_declaration_ref,
                        external_assignment_ref,
                    );
                });

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
                let input = pool_with(|pool| {
                    let left_ref = builders::primitive_integer(pool, left);
                    let right_ref = builders::primitive_integer(pool, right);
                    builders::infix(pool, operation, left_ref, right_ref);
                });

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }
}
