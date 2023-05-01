use std::rc::Rc;

use crate::ast::*;
use crate::primitive::*;

pub fn interpret<Annotation: Clone>(expr: Rc<Expr<Annotation>>) -> Rc<Expr<Annotation>> {
    match expr.as_ref() {
        Expr::Primitive { .. } => expr,
        Expr::Infix {
            annotation,
            operation,
            left,
            right,
        } => match (left.as_ref(), right.as_ref()) {
            (
                Expr::Primitive {
                    annotation: _,
                    value: Primitive::Int(left),
                },
                Expr::Primitive {
                    annotation: _,
                    value: Primitive::Int(right),
                },
            ) => match *operation {
                Operation::Add => Expr::Primitive {
                    annotation: annotation.clone(),
                    value: Primitive::Int(left + right),
                },
                Operation::Subtract => Expr::Primitive {
                    annotation: annotation.clone(),
                    value: Primitive::Int(left - right),
                },
                Operation::Multiply => Expr::Primitive {
                    annotation: annotation.clone(),
                    value: Primitive::Int(left * right),
                },
            }
            .into(),
            _ => {
                let left_result = interpret(left.clone());
                let right_result = interpret(right.clone());
                interpret(
                    Expr::Infix {
                        annotation: annotation.clone(),
                        operation: *operation,
                        left: left_result,
                        right: right_result,
                    }
                    .into(),
                )
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreting_a_primitive() {
        arbtest::builder().run(|u| {
            let value = u.arbitrary::<Primitive>()?;
            let expr = Rc::new(Expr::Primitive {
                annotation: (),
                value,
            });
            let result = interpret(expr.clone());
            assert_eq!(result, expr);
            Ok(())
        })
    }

    #[test]
    fn test_interpreting_addition() {
        test_interpreting_an_operation(Operation::Add, Int::checked_add)
    }

    #[test]
    fn test_interpreting_subraction() {
        test_interpreting_an_operation(Operation::Subtract, Int::checked_sub)
    }

    #[test]
    fn test_interpreting_multiplication() {
        test_interpreting_an_operation(Operation::Multiply, Int::checked_mul)
    }

    fn test_interpreting_an_operation(
        operation: Operation,
        implementation: impl Fn(Int, Int) -> Option<Int>,
    ) {
        arbtest::builder().run(|u| {
            let left = u.int_in_range(i32::MIN..=i32::MAX).map(|x| x.into())?;
            let right = u.int_in_range(i32::MAX..=i32::MAX).map(|x| x.into())?;
            match implementation(left, right) {
                None => Ok(()), // overflow or underflow
                Some(expected) => {
                    let expr = Expr::Infix {
                        annotation: (),
                        operation,
                        left: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(left),
                        }
                        .into(),
                        right: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(right),
                        }
                        .into(),
                    };
                    let result = interpret(expr.into());
                    assert_eq!(
                        *result,
                        Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(expected),
                        }
                    );
                    Ok(())
                }
            }
        })
    }
}
