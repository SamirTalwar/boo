use std::rc::Rc;

use im::HashMap;

use crate::ast::*;
use crate::primitive::*;

pub fn interpret<Annotation: Clone>(expr: Rc<Expr<Annotation>>) -> Rc<Expr<Annotation>> {
    interpret_(expr, HashMap::new())
}

pub fn interpret_<'a, Annotation: Clone>(
    expr: Rc<Expr<'a, Annotation>>,
    assignments: HashMap<&'a str, Rc<Expr<'a, Annotation>>>,
) -> Rc<Expr<'a, Annotation>> {
    match expr.as_ref() {
        Expr::Primitive { .. } => expr,
        Expr::Identifier {
            annotation: _,
            name,
        } => match assignments.get(name) {
            Some(value) => interpret_(value.clone(), assignments),
            None => todo!(),
        },
        Expr::Let {
            annotation: _,
            name,
            value,
            inner,
        } => interpret_(inner.clone(), assignments.update(*name, value.clone())),
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
                let left_result = interpret_(left.clone(), assignments.clone());
                let right_result = interpret_(right.clone(), assignments.clone());
                interpret_(
                    Expr::Infix {
                        annotation: annotation.clone(),
                        operation: *operation,
                        left: left_result,
                        right: right_result,
                    }
                    .into(),
                    assignments.clone(),
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
    fn test_interpreting_assignment() {
        arbtest::builder().run(|u| {
            let name = "variable";
            let value = u.arbitrary::<Primitive>()?;
            let expr = Expr::Let {
                annotation: (),
                name,
                value: Expr::Primitive {
                    annotation: (),
                    value,
                }
                .into(),
                inner: Expr::Identifier {
                    annotation: (),
                    name,
                }
                .into(),
            };
            let result = interpret(expr.into());
            assert_eq!(
                *result,
                Expr::Primitive {
                    annotation: (),
                    value,
                }
            );
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

    #[test]
    fn test_interpreting_variable_use() {
        arbtest::builder().run(|u| {
            let name = "variable";
            let variable = u.arbitrary::<Int>()?;
            let constant = u.arbitrary::<Int>()?;
            match variable.checked_add(constant) {
                None => Ok(()), // overflow or underflow
                Some(sum) => {
                    let expr = Rc::new(Expr::Let {
                        annotation: (),
                        name,
                        value: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(variable),
                        }
                        .into(),
                        inner: Expr::Infix {
                            annotation: (),
                            operation: Operation::Add,
                            left: Expr::Identifier {
                                annotation: (),
                                name,
                            }
                            .into(),
                            right: Expr::Primitive {
                                annotation: (),
                                value: Primitive::Int(constant),
                            }
                            .into(),
                        }
                        .into(),
                    });
                    let result = interpret(expr.clone());
                    assert_eq!(
                        *result,
                        Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(sum),
                        }
                    );
                    Ok(())
                }
            }
        })
    }
}
