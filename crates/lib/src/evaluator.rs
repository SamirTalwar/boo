use std::borrow::Cow;
use std::sync::Arc;

use im::HashMap;

use crate::error::*;
use crate::identifier::*;
use crate::operation::*;
use crate::pooler::ast::*;
use crate::primitive::*;
use crate::thunk::Thunk;

pub fn evaluate(pool: &ExprPool) -> Result<Expression> {
    let evaluated = evaluate_(pool, pool.root(), HashMap::new())?;
    Ok(evaluated.into_owned())
}

fn evaluate_<'a>(
    pool: &'a ExprPool,
    expr_ref: Expr,
    assignments: HashMap<&'a Identifier, Thunk<Expr, Result<Cow<'a, Expression>>>>,
) -> Result<Cow<'a, Expression>> {
    let expr = pool.get(expr_ref);
    match &expr.value {
        value @ Expression::Primitive(_) => Ok(Cow::Borrowed(value)),
        Expression::Identifier(name) => match assignments.clone().get_mut(name) {
            Some(value_ref) => {
                let result = value_ref.resolve_by(|r| evaluate_(pool, *r, assignments));
                Arc::try_unwrap(result).unwrap_or_else(|arc| (*arc).clone())
            }
            None => Err(Error::UnknownVariable {
                span: expr.span,
                name: name.to_string(),
            }),
        },
        Expression::Assign(Assign {
            name,
            value: value_ref,
            inner: inner_ref,
        }) => evaluate_(
            pool,
            *inner_ref,
            assignments.update(name, Thunk::unresolved(*value_ref)),
        ),
        Expression::Infix(Infix {
            operation,
            left: left_ref,
            right: right_ref,
        }) => {
            let left_result = evaluate_(pool, *left_ref, assignments.clone())?;
            let right_result = evaluate_(pool, *right_ref, assignments)?;
            Ok(evaluate_infix(*operation, left_result, right_result))
        }
    }
}

fn evaluate_infix<'a>(
    operation: Operation,
    left: Cow<'a, Expression>,
    right: Cow<'a, Expression>,
) -> Cow<'a, Expression> {
    match (left.as_ref(), right.as_ref()) {
        (
            Expression::Primitive(Primitive::Integer(left)),
            Expression::Primitive(Primitive::Integer(right)),
        ) => Cow::Owned(Expression::Primitive(match operation {
            Operation::Add => Primitive::Integer(left + right),
            Operation::Subtract => Primitive::Integer(left - right),
            Operation::Multiply => Primitive::Integer(left * right),
        })),
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
    use crate::pooler::pool::pool_with;
    use crate::span::Spanned;

    use super::*;

    #[test]
    fn test_evaluating_a_primitive() {
        check(&Primitive::arbitrary(), |value| {
            let input = pool_with(|pool| {
                builders::primitive(pool, value.clone());
            });
            let expected = Expression::Primitive(value);

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
                let expected = Expression::Primitive(value);

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
                    Expression::Primitive(Primitive::Integer(implementation(&left, &right)));
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
                let expected = Expression::Primitive(Primitive::Integer(sum));

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }
}
