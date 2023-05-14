use std::rc::Rc;

use im::HashMap;

use crate::ast::*;
use crate::error::*;
use crate::identifier::*;
use crate::primitive::*;
use crate::span::*;

pub fn evaluate(expr: Rc<Expr<Span>>) -> Result<Rc<Expr<()>>> {
    evaluate_(expr, HashMap::new())
}

pub fn evaluate_(
    expr: Rc<Expr<Span>>,
    assignments: HashMap<Identifier, Rc<Expr<Span>>>,
) -> Result<Rc<Expr<()>>> {
    match expr.as_ref() {
        Expr::Primitive {
            annotation: _,
            value,
        } => Ok(Expr::Primitive {
            annotation: (),
            value: value.clone(),
        }
        .into()),
        Expr::Identifier { annotation, name } => match assignments.get(name) {
            Some(value) => evaluate_(value.clone(), assignments),
            None => Err(Error::UnknownVariable {
                span: *annotation,
                name: name.to_string(),
            }),
        },
        Expr::Let {
            annotation: _,
            name,
            value,
            inner,
        } => evaluate_(
            inner.clone(),
            assignments.update(name.clone(), value.clone()),
        ),
        Expr::Infix {
            annotation: _,
            operation,
            left,
            right,
        } => match (left.as_ref(), right.as_ref()) {
            (Expr::Primitive { .. }, Expr::Primitive { .. }) => {
                Ok(evaluate_infix(*operation, left.as_ref(), right.as_ref()))
            }
            _ => {
                let left_result = evaluate_(left.clone(), assignments.clone())?;
                let right_result = evaluate_(right.clone(), assignments)?;
                Ok(evaluate_infix(
                    *operation,
                    left_result.as_ref(),
                    right_result.as_ref(),
                ))
            }
        },
    }
}

fn evaluate_infix<Annotation>(
    operation: Operation,
    left: &Expr<Annotation>,
    right: &Expr<Annotation>,
) -> Rc<Expr<()>> {
    match (left, right) {
        (
            Expr::Primitive {
                annotation: _,
                value: Primitive::Integer(left),
            },
            Expr::Primitive {
                annotation: _,
                value: Primitive::Integer(right),
            },
        ) => match operation {
            Operation::Add => Expr::Primitive {
                annotation: (),
                value: Primitive::Integer(left + right),
            },
            Operation::Subtract => Expr::Primitive {
                annotation: (),
                value: Primitive::Integer(left - right),
            },
            Operation::Multiply => Expr::Primitive {
                annotation: (),
                value: Primitive::Integer(left * right),
            },
        }
        .into(),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::ast::builders::*;
    use crate::proptest_helpers::*;

    use super::*;

    #[test]
    fn test_interpreting_a_primitive() {
        check(&Primitive::arbitrary(), |value| {
            let input = primitive(0, value.clone());
            let expected = primitive((), value);

            let actual = evaluate(input.into());

            prop_assert_eq!(actual, Ok(expected.into()));
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

                let actual = evaluate(input.into());

                prop_assert_eq!(actual, Ok(expected.into()));
                Ok(())
            },
        )
    }

    #[test]
    fn test_interpreting_an_unknown_variable() {
        check(&Identifier::arbitrary(), |name| {
            let input = identifier(5..10, name.clone());

            let actual = evaluate(input.into());

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

                let actual = evaluate(input.into());

                prop_assert_eq!(actual, Ok(expected.into()));
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
                let expected = Expr::Primitive {
                    annotation: (),
                    value: Primitive::Integer(sum),
                };

                let actual = evaluate(input.into());

                prop_assert_eq!(actual, Ok(expected.into()));
                Ok(())
            },
        )
    }
}
