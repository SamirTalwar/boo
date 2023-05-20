use im::HashMap;

use crate::error::*;
use crate::identifier::*;
use crate::operation::*;
use crate::parser::ast::*;
use crate::primitive::*;
use crate::span::*;

pub fn evaluate(expr: Expr<Span>) -> Result<Expr<()>> {
    evaluate_(expr, HashMap::new())
}

pub fn evaluate_(
    expr: Expr<Span>,
    assignments: HashMap<Identifier, Expr<Span>>,
) -> Result<Expr<()>> {
    match &expr.value {
        Expression::Primitive { value } => Ok(Annotated {
            annotation: (),
            value: Expression::Primitive {
                value: value.clone(),
            },
        }
        .into()),
        Expression::Identifier { name } => match assignments.get(name) {
            Some(value) => evaluate_(value.clone(), assignments),
            None => Err(Error::UnknownVariable {
                span: expr.annotation,
                name: name.to_string(),
            }),
        },
        Expression::Let { name, value, inner } => evaluate_(
            inner.clone(),
            assignments.update(name.clone(), value.clone()),
        ),
        Expression::Infix {
            operation,
            left,
            right,
        } => match (&left.value, &right.value) {
            (Expression::Primitive { .. }, Expression::Primitive { .. }) => {
                Ok(evaluate_infix(*operation, left.clone(), right.clone()))
            }
            _ => {
                let left_result = evaluate_(left.clone(), assignments.clone())?;
                let right_result = evaluate_(right.clone(), assignments)?;
                Ok(evaluate_infix(*operation, left_result, right_result))
            }
        },
    }
}

fn evaluate_infix<Annotation>(
    operation: Operation,
    left: Expr<Annotation>,
    right: Expr<Annotation>,
) -> Expr<()> {
    let value = match (&left.value, &right.value) {
        (
            Expression::Primitive {
                value: Primitive::Integer(left),
            },
            Expression::Primitive {
                value: Primitive::Integer(right),
            },
        ) => match operation {
            Operation::Add => Expression::Primitive {
                value: Primitive::Integer(left + right),
            },
            Operation::Subtract => Expression::Primitive {
                value: Primitive::Integer(left - right),
            },
            Operation::Multiply => Expression::Primitive {
                value: Primitive::Integer(left * right),
            },
        },
        _ => unreachable!(),
    };
    Annotated {
        annotation: (),
        value,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::parser::builders::*;
    use crate::proptest_helpers::*;

    use super::*;

    #[test]
    fn test_interpreting_a_primitive() {
        check(&Primitive::arbitrary(), |value| {
            let input = primitive(0, value.clone());
            let expected = primitive((), value);

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
                let input = assign(
                    0,
                    name.clone(),
                    primitive(0, value.clone()),
                    identifier(0, name),
                );
                let expected = primitive((), value);

                let actual = evaluate(input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_interpreting_an_unknown_variable() {
        check(&Identifier::arbitrary(), |name| {
            let input = identifier(5..10, name.clone());

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
                let expected = primitive_integer((), implementation(&left, &right));
                let input = infix(
                    0,
                    operation,
                    primitive_integer(0, left),
                    primitive_integer(0, right),
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
                let input = assign(
                    0,
                    name.clone(),
                    primitive_integer(0, variable),
                    infix(
                        0,
                        Operation::Add,
                        identifier(0, name),
                        primitive_integer(0, constant),
                    ),
                );
                let expected = primitive_integer((), sum);

                let actual = evaluate(input);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }
}
