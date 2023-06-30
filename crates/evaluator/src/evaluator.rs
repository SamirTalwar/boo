//! Evaluates a [pooled `Expr`][super::pooler::ast::Expr].

use std::borrow::Cow;
use std::sync::Arc;

use im::HashMap;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::identifier::*;
use boo_core::native::*;
use boo_core::operation::*;
use boo_core::primitive::*;
use boo_core::span::Span;

use crate::pooler::ast::*;
use crate::thunk::Thunk;

/// Evaluate a [pooled `Expr`][super::pooler::ast::Expr].
pub fn evaluate(pool: &ExprPool, root: Expr) -> Result<Evaluated> {
    Evaluator {
        pool,
        bindings: Bindings::new(),
    }
    .evaluate(root)
    .map(|progress| progress.finish())
}

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

    pub fn read(
        &mut self,
        identifier: &Identifier,
    ) -> Option<&mut Thunk<UnevaluatedBinding<'a>, EvaluatedBinding<'a>>> {
        self.0.get_mut(identifier)
    }

    /// Adds a new binding to the set.
    pub fn with(
        &self,
        identifier: &'a Identifier,
        expression: Expr,
        expression_bindings: Self,
    ) -> Self {
        Self(self.0.update(
            Cow::Borrowed(identifier),
            Thunk::unresolved((expression, expression_bindings)),
        ))
    }
}

/// An expression pool together with the current bound context, which can
/// evaluate a given expression reference from the pool.
struct Evaluator<'a> {
    pool: &'a ExprPool,
    bindings: Bindings<'a>,
}

impl<'a> Evaluator<'a> {
    /// Evaluates an expression from a pool in a given scope.
    ///
    /// The bindings are modified by assignment, accessed when evaluating an
    /// identifier, and captured by closures when a function is evaluated.
    pub fn evaluate(&self, expr_ref: Expr) -> Result<EvaluationProgress<'a>> {
        let expr = expr_ref.read_from(self.pool);
        match &expr.value {
            Expression::Primitive(value) => Ok(EvaluationProgress::Primitive(Cow::Borrowed(value))),
            Expression::Native(Native { implementation, .. }) => {
                implementation(self).map(|value| EvaluationProgress::Primitive(Cow::Owned(value)))
            }
            Expression::Identifier(name) => self.resolve(name, expr.span),
            Expression::Assign(Assign {
                name,
                value: value_ref,
                inner: inner_ref,
            }) => self.with(name, *value_ref).evaluate(*inner_ref),
            Expression::Function(function) => {
                Ok(EvaluationProgress::Closure(function, self.bindings.clone()))
            }
            Expression::Apply(Apply {
                function: function_ref,
                argument: argument_ref,
            }) => {
                let function_result = self.evaluate(*function_ref)?;
                match function_result {
                    EvaluationProgress::Closure(
                        Function {
                            parameter,
                            body: body_ref,
                        },
                        function_bindings,
                    ) => self
                        // the body is executed in the context of the function,
                        // but the argument must be evaluated in the outer context
                        .switch(function_bindings.with(
                            parameter,
                            *argument_ref,
                            self.bindings.clone(),
                        ))
                        .evaluate(*body_ref),
                    _ => Err(Error::InvalidFunctionApplication { span: expr.span }),
                }
            }
            Expression::Infix(Infix {
                operation,
                left: left_ref,
                right: right_ref,
            }) => {
                let left_result = self.evaluate(*left_ref)?;
                let right_result = self.evaluate(*right_ref)?;
                match (&left_result, &right_result) {
                    (EvaluationProgress::Primitive(left), EvaluationProgress::Primitive(right)) => {
                        match (left.as_ref(), right.as_ref()) {
                            (Primitive::Integer(left), Primitive::Integer(right)) => {
                                Ok(EvaluationProgress::Primitive(Cow::Owned(match operation {
                                    Operation::Add => Primitive::Integer(left + right),
                                    Operation::Subtract => Primitive::Integer(left - right),
                                    Operation::Multiply => Primitive::Integer(left * right),
                                })))
                            }
                        }
                    }
                    _ => Err(Error::TypeError),
                }
            }
        }
    }

    /// Resolves a given identifier by evaluating its binding.
    fn resolve(&self, identifier: &Identifier, span: Span) -> EvaluatedBinding<'a> {
        match self.bindings.clone().read(identifier) {
            Some(thunk) => {
                let result = thunk.resolve_by(move |(value_ref, thunk_bindings)| {
                    self.switch(thunk_bindings.clone()).evaluate(*value_ref)
                });
                Arc::try_unwrap(result).unwrap_or_else(|arc| (*arc).clone())
            }
            None => Err(Error::UnknownVariable {
                span,
                name: identifier.to_string(),
            }),
        }
    }

    fn with(&self, identifier: &'a Identifier, expression: Expr) -> Self {
        self.switch(
            self.bindings
                .with(identifier, expression, self.bindings.clone()),
        )
    }

    fn switch(&self, new_bindings: Bindings<'a>) -> Self {
        Self {
            pool: self.pool,
            bindings: new_bindings,
        }
    }
}

impl<'a> NativeContext for Evaluator<'a> {
    fn lookup_value(&self, identifier: &Identifier) -> Result<Primitive> {
        match self.resolve(identifier, 0.into())?.finish() {
            Evaluated::Primitive(primitive) => Ok(primitive),
            Evaluated::Function(_) => Err(Error::TypeError),
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use boo_core::ast::builders;
    use boo_test_helpers::proptest::*;

    use crate::pooler::pool::pool_with;

    use super::*;

    fn pool_of(expr: simple::Expr) -> (ExprPool, Expr) {
        pool_with(|pool| {
            expr.transform(&mut |_, expression| Expr::insert(pool, 0.into(), expression))
        })
    }

    #[test]
    fn test_evaluating_a_primitive() {
        check(&Primitive::arbitrary(), |value| {
            // input: `value`
            let (input, root) = pool_of(builders::primitive((), value.clone()));
            let expected = Evaluated::Primitive(value);

            let actual = evaluate(&input, root);

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
                let (input, root) = pool_of(builders::assign(
                    (),
                    name.clone(),
                    builders::primitive((), value.clone()),
                    builders::identifier((), name),
                ));
                let expected = Evaluated::Primitive(value);

                let actual = evaluate(&input, root);

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
                let (input, root) = pool_of(builders::assign(
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

                let actual = evaluate(&input, root);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_evaluating_an_unknown_variable() {
        check(&Identifier::arbitrary(), |name| {
            let (input, root) = pool_with(|pool| {
                Expr::insert(pool, (5..10).into(), Expression::Identifier(name.clone()))
            });

            let actual = evaluate(&input, root);

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
            let (input, (root, body_ref)) = pool_with(|pool| {
                let body_ref = crate::pooler::builders::identifier(pool, parameter.clone());
                let root = crate::pooler::builders::function(pool, parameter.clone(), body_ref);
                (root, body_ref)
            });
            let expected = Evaluated::Function(Function {
                parameter,
                body: body_ref,
            });

            let actual = evaluate(&input, root);

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
                let (input, root) = pool_of(builders::apply(
                    (),
                    builders::function((), parameter.clone(), builders::identifier((), parameter)),
                    builders::primitive_integer((), argument.clone()),
                ));
                let expected = Evaluated::Primitive(Primitive::Integer(argument));

                let actual = evaluate(&input, root);

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
                let (input, root) = pool_of(builders::apply(
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

                let actual = evaluate(&input, root);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_function_application_with_named_argument() {
        check(
            &(
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Identifier::arbitrary(),
            ),
            |(argument_name, argument, parameter)| {
                // input: let `argument_name` = `argument` in (fn `parameter` -> `parameter`) `argument_name`
                let (input, root) = pool_of(builders::assign(
                    (),
                    argument_name.clone(),
                    builders::primitive_integer((), argument.clone()),
                    builders::apply(
                        (),
                        builders::function(
                            (),
                            parameter.clone(),
                            builders::identifier((), parameter),
                        ),
                        builders::identifier((), argument_name),
                    ),
                ));
                let expected = Evaluated::Primitive(Primitive::Integer(argument));

                let actual = evaluate(&input, root);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_named_function_application() {
        check(
            &(
                Identifier::arbitrary(),
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(function_name, parameter, multiplier, argument)| {
                // input:
                //   let `function_name` = (fn `parameter` -> `parameter` * `multiplier`)
                //     in (`function_name` `argument`)
                let (input, root) = pool_of(builders::assign(
                    (),
                    function_name.clone(),
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
                    builders::apply(
                        (),
                        builders::identifier((), function_name),
                        builders::primitive_integer((), argument.clone()),
                    ),
                ));
                let expected = Evaluated::Primitive(Primitive::Integer(argument * multiplier));

                let actual = evaluate(&input, root);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_named_function_application_with_named_argument() {
        check(
            &(
                Identifier::arbitrary(),
                Identifier::arbitrary(),
                Identifier::arbitrary(),
                Integer::arbitrary(),
            ),
            |(function_name, parameter, argument_name, argument)| {
                // input: let `function_name` = (fn `parameter` -> `parameter`) in let `argument_name` = `argument` in `function_name` `argument_name`
                let (input, root) = pool_of(builders::assign(
                    (),
                    function_name.clone(),
                    builders::function((), parameter.clone(), builders::identifier((), parameter)),
                    builders::assign(
                        (),
                        argument_name.clone(),
                        builders::primitive_integer((), argument.clone()),
                        builders::apply(
                            (),
                            builders::identifier((), function_name),
                            builders::identifier((), argument_name),
                        ),
                    ),
                ));
                let expected = Evaluated::Primitive(Primitive::Integer(argument));

                let actual = evaluate(&input, root);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_named_function_application_with_multiple_parameters() {
        check(
            &(
                Identifier::arbitrary(),
                Identifier::arbitrary(),
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(function_name, left, right, a, b)| {
                // input:
                //   let `function_name` = (fn `left` -> fn `right` -> `left` * `right`)
                //     in (`function_name` `a` `b`)
                let (input, root) = pool_of(builders::assign(
                    (),
                    function_name.clone(),
                    builders::function(
                        (),
                        left.clone(),
                        builders::function(
                            (),
                            right.clone(),
                            builders::infix(
                                (),
                                Operation::Multiply,
                                builders::identifier((), left),
                                builders::identifier((), right),
                            ),
                        ),
                    ),
                    builders::apply(
                        (),
                        builders::apply(
                            (),
                            builders::identifier((), function_name),
                            builders::primitive_integer((), a.clone()),
                        ),
                        builders::primitive_integer((), b.clone()),
                    ),
                ));
                let expected = Evaluated::Primitive(Primitive::Integer(a * b));

                let actual = evaluate(&input, root);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_named_function_application_nested() {
        check(
            &(
                Identifier::arbitrary(),
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(function_name, parameter, multiplier, argument)| {
                // input:
                //   let `function_name` = (fn `parameter` -> `parameter` * `multiplier`)
                //     in (`function_name` (`function_name` `argument`))
                let (input, root) = pool_of(builders::assign(
                    (),
                    function_name.clone(),
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
                    builders::apply(
                        (),
                        builders::identifier((), function_name.clone()),
                        builders::apply(
                            (),
                            builders::identifier((), function_name),
                            builders::primitive_integer((), argument.clone()),
                        ),
                    ),
                ));
                let expected = Evaluated::Primitive(Primitive::Integer(
                    argument * multiplier.clone() * multiplier,
                ));

                let actual = evaluate(&input, root);

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
                let (input, root) = pool_of(builders::apply(
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

                let actual = evaluate(&input, root);

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
                let (input, root) = pool_of(builders::assign(
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

                let actual = evaluate(&input, root);

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
                let (input, root) = pool_of(builders::infix(
                    (),
                    operation,
                    builders::primitive_integer((), left),
                    builders::primitive_integer((), right),
                ));

                let actual = evaluate(&input, root);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }
}
