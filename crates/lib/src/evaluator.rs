use std::borrow::Cow;

use im::HashMap;

use crate::error::*;
use crate::identifier::*;
use crate::operation::*;
use crate::pooler;
use crate::pooler::ast::ExprPool;
use crate::primitive::*;

pub fn evaluate(pool: &ExprPool) -> Result<Primitive> {
    evaluate_(pool, pool.root(), HashMap::new()).map(Cow::into_owned)
}

pub fn evaluate_<'a>(
    pool: &'a ExprPool,
    expr_ref: pooler::ast::Expr,
    assignments: HashMap<&'a Identifier, pooler::ast::Expr>,
) -> Result<Cow<'a, Primitive>> {
    let expr = pool.get(expr_ref);
    match &expr.value {
        pooler::ast::Expression::Primitive { value } => Ok(Cow::Borrowed(value)),
        pooler::ast::Expression::Identifier { name } => match assignments.get(name) {
            Some(value_ref) => evaluate_(pool, *value_ref, assignments),
            None => Err(Error::UnknownVariable {
                span: expr.span,
                name: name.to_string(),
            }),
        },
        pooler::ast::Expression::Assign {
            name,
            value: value_ref,
            inner: inner_ref,
        } => evaluate_(pool, *inner_ref, assignments.update(name, *value_ref)),
        pooler::ast::Expression::Infix {
            operation,
            left: left_ref,
            right: right_ref,
        } => match (&pool.get(*left_ref).value, &pool.get(*right_ref).value) {
            (
                pooler::ast::Expression::Primitive { value: left_value },
                pooler::ast::Expression::Primitive { value: right_value },
            ) => Ok(Cow::Owned(evaluate_infix(
                *operation,
                left_value,
                right_value,
            ))),
            _ => {
                let left_result = evaluate_(pool, *left_ref, assignments.clone())?;
                let right_result = evaluate_(pool, *right_ref, assignments)?;
                Ok(Cow::Owned(evaluate_infix(
                    *operation,
                    &left_result,
                    &right_result,
                )))
            }
        },
    }
}

fn evaluate_infix<'a>(
    operation: Operation,
    left: &'a Primitive,
    right: &'a Primitive,
) -> Primitive {
    match (&left, &right) {
        (Primitive::Integer(left), Primitive::Integer(right)) => match operation {
            Operation::Add => Primitive::Integer(left + right),
            Operation::Subtract => Primitive::Integer(left - right),
            Operation::Multiply => Primitive::Integer(left * right),
        },
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::pooler::pool::pool_with;
    use crate::proptest_helpers::*;
    use crate::span::Spanned;

    use super::*;

    #[test]
    fn test_interpreting_a_primitive() {
        check(&Primitive::arbitrary(), |value| {
            let input = pool_with(|pool| {
                pooler::builders::primitive(pool, value.clone());
            });
            let expected = value;

            let actual = evaluate(&input);

            prop_assert_eq!(actual, Ok(expected));
            Ok(())
        })
    }

    #[test]
    fn test_interpreting_assignment() {
        check(
            &(Identifier::arbitrary(), Primitive::arbitrary()),
            |(name, value)| {
                let input = pool_with(|pool| {
                    let value_ref = pooler::builders::primitive(pool, value.clone());
                    let inner_ref = pooler::builders::identifier(pool, name.clone());
                    pooler::builders::assign(pool, name.clone(), value_ref, inner_ref);
                });
                let expected = value;

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_interpreting_an_unknown_variable() {
        check(&Identifier::arbitrary(), |name| {
            let input = pool_with(|pool| {
                pool.add(Spanned {
                    span: (5..10).into(),
                    value: pooler::ast::Expression::Identifier { name: name.clone() },
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
    fn test_interpreting_addition() {
        test_interpreting_an_operation(Operation::Add, |x, y| x + y)
    }

    #[test]
    fn test_interpreting_subtraction() {
        test_interpreting_an_operation(Operation::Subtract, |x, y| x - y)
    }

    #[test]
    fn test_interpreting_multiplication() {
        test_interpreting_an_operation(Operation::Multiply, |x, y| x * y)
    }

    fn test_interpreting_an_operation(
        operation: Operation,
        implementation: impl Fn(&Integer, &Integer) -> Integer,
    ) {
        check(
            &(Integer::arbitrary(), Integer::arbitrary()),
            |(left, right)| {
                let expected = Primitive::Integer(implementation(&left, &right));
                let input = pool_with(|pool| {
                    let left_ref = pooler::builders::primitive_integer(pool, left);
                    let right_ref = pooler::builders::primitive_integer(pool, right);
                    pooler::builders::infix(pool, operation, left_ref, right_ref);
                });

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_interpreting_variable_use() {
        check(
            &(
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(name, variable, constant)| {
                let sum = &variable + &constant;
                let input = pool_with(|pool| {
                    let left_ref = pooler::builders::identifier(pool, name.clone());
                    let right_ref = pooler::builders::primitive_integer(pool, constant);
                    let value_ref = pooler::builders::primitive_integer(pool, variable);
                    let inner_ref =
                        pooler::builders::infix(pool, Operation::Add, left_ref, right_ref);
                    pooler::builders::assign(pool, name.clone(), value_ref, inner_ref);
                });
                let expected = Primitive::Integer(sum);

                let actual = evaluate(&input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }
}
