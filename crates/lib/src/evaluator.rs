pub mod ast;
pub mod builders;

use im::HashMap;

use crate::error::*;
use crate::identifier::*;
use crate::operation::*;
use crate::parser;
use crate::primitive::*;
use crate::span::*;

pub fn evaluate(expr: parser::ast::Expr<Span>) -> Result<ast::Expr> {
    evaluate_(expr, HashMap::new())
}

pub fn evaluate_(
    expr: parser::ast::Expr<Span>,
    assignments: HashMap<Identifier, parser::ast::Expr<Span>>,
) -> Result<ast::Expr> {
    match &expr.value {
        parser::ast::Expression::Primitive { value } => Ok(ast::Expression::Primitive {
            value: value.clone(),
        }
        .into()),
        parser::ast::Expression::Identifier { name } => match assignments.get(name) {
            Some(value) => evaluate_(value.clone(), assignments),
            None => Err(Error::UnknownVariable {
                span: expr.annotation,
                name: name.to_string(),
            }),
        },
        parser::ast::Expression::Let { name, value, inner } => evaluate_(
            inner.clone(),
            assignments.update(name.clone(), value.clone()),
        ),
        parser::ast::Expression::Infix {
            operation,
            left,
            right,
        } => match (&left.value, &right.value) {
            (
                parser::ast::Expression::Primitive { value: left_value },
                parser::ast::Expression::Primitive { value: right_value },
            ) => Ok(evaluate_infix(
                *operation,
                ast::Expression::Primitive {
                    value: left_value.clone(),
                }
                .into(),
                ast::Expression::Primitive {
                    value: right_value.clone(),
                }
                .into(),
            )),
            _ => {
                let left_result = evaluate_(left.clone(), assignments.clone())?;
                let right_result = evaluate_(right.clone(), assignments)?;
                Ok(evaluate_infix(*operation, left_result, right_result))
            }
        },
    }
}

fn evaluate_infix(operation: Operation, left: ast::Expr, right: ast::Expr) -> ast::Expr {
    let value = match (left.as_ref(), right.as_ref()) {
        (
            ast::Expression::Primitive {
                value: Primitive::Integer(left),
            },
            ast::Expression::Primitive {
                value: Primitive::Integer(right),
            },
        ) => match operation {
            Operation::Add => ast::Expression::Primitive {
                value: Primitive::Integer(left + right),
            },
            Operation::Subtract => ast::Expression::Primitive {
                value: Primitive::Integer(left - right),
            },
            Operation::Multiply => ast::Expression::Primitive {
                value: Primitive::Integer(left * right),
            },
        },
        _ => unreachable!(),
    };
    value.into()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::proptest_helpers::*;
    use builders::*;

    use super::*;

    #[test]
    fn test_interpreting_a_primitive() {
        check(&Primitive::arbitrary(), |value| {
            let input = parser::builders::primitive(0, value.clone());
            let expected = primitive(value);

            let actual = evaluate(input);

            prop_assert_eq!(actual, Ok(expected));
            Ok(())
        })
    }

    #[test]
    fn test_interpreting_assignment() {
        check(
            &(Identifier::arbitrary(), Primitive::arbitrary()),
            |(name, value)| {
                let input = parser::builders::assign(
                    0,
                    name.clone(),
                    parser::builders::primitive(0, value.clone()),
                    parser::builders::identifier(0, name),
                );
                let expected = primitive(value);

                let actual = evaluate(input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_interpreting_an_unknown_variable() {
        check(&Identifier::arbitrary(), |name| {
            let input = parser::builders::identifier(5..10, name.clone());

            let actual = evaluate(input);

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
                let expected = primitive_integer(implementation(&left, &right));
                let input = parser::builders::infix(
                    0,
                    operation,
                    parser::builders::primitive_integer(0, left),
                    parser::builders::primitive_integer(0, right),
                );

                let actual = evaluate(input);

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
                let input = parser::builders::assign(
                    0,
                    name.clone(),
                    parser::builders::primitive_integer(0, variable),
                    parser::builders::infix(
                        0,
                        Operation::Add,
                        parser::builders::identifier(0, name),
                        parser::builders::primitive_integer(0, constant),
                    ),
                );
                let expected = primitive_integer(sum);

                let actual = evaluate(input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }
}
