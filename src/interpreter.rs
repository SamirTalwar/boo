use crate::ast::*;
use crate::primitive::*;

pub fn interpret<Annotation>(expr: Expr<Annotation>) -> Expr<Annotation> {
    match expr {
        expr @ Expr::Primitive { .. } => expr,
        Expr::Infix {
            annotation,
            operation,
            left,
            right,
        } => match (*left, *right) {
            (
                Expr::Primitive {
                    annotation: _,
                    value: Primitive::Int(left),
                },
                Expr::Primitive {
                    annotation: _,
                    value: Primitive::Int(right),
                },
            ) => match operation {
                Operation::Add => Expr::Primitive {
                    annotation,
                    value: Primitive::Int(left + right),
                },
                Operation::Subtract => Expr::Primitive {
                    annotation,
                    value: Primitive::Int(left - right),
                },
                Operation::Multiply => Expr::Primitive {
                    annotation,
                    value: Primitive::Int(left * right),
                },
            },
            (left, right) => {
                let left_result = interpret(left);
                let right_result = interpret(right);
                interpret(Expr::Infix {
                    annotation,
                    operation,
                    left: Box::new(left_result),
                    right: Box::new(right_result),
                })
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
            let expr = Expr::Primitive {
                annotation: (),
                value,
            };
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
                        left: Box::new(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(left),
                        }),
                        right: Box::new(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(right),
                        }),
                    };
                    let result = interpret(expr);
                    assert_eq!(
                        result,
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
