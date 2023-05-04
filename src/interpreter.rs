use std::rc::Rc;

use im::HashMap;

use crate::ast::*;
use crate::error::*;
use crate::identifier::*;
use crate::primitive::*;

pub fn interpret<Annotation: Clone>(expr: Rc<Expr<Annotation>>) -> Result<Rc<Expr<Annotation>>> {
    interpret_(expr, HashMap::new())
}

pub fn interpret_<'a, Annotation: Clone>(
    expr: Rc<Expr<'a, Annotation>>,
    assignments: HashMap<Identifier<'a>, Rc<Expr<'a, Annotation>>>,
) -> Result<Rc<Expr<'a, Annotation>>> {
    match expr.as_ref() {
        Expr::Primitive { .. } => Ok(expr),
        Expr::Identifier {
            annotation: _,
            name,
        } => match assignments.get(name) {
            Some(value) => interpret_(value.clone(), assignments),
            None => Err(Error::UnknownVariable {
                span: 0.into(), // this is wrong
                name: name.to_string(),
            }),
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
                    value: Primitive::Integer(left),
                },
                Expr::Primitive {
                    annotation: _,
                    value: Primitive::Integer(right),
                },
            ) => Ok(match *operation {
                Operation::Add => Expr::Primitive {
                    annotation: annotation.clone(),
                    value: Primitive::Integer(left + right),
                },
                Operation::Subtract => Expr::Primitive {
                    annotation: annotation.clone(),
                    value: Primitive::Integer(left - right),
                },
                Operation::Multiply => Expr::Primitive {
                    annotation: annotation.clone(),
                    value: Primitive::Integer(left * right),
                },
            }
            .into()),
            _ => {
                let left_result = interpret_(left.clone(), assignments.clone())?;
                let right_result = interpret_(right.clone(), assignments.clone())?;
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
            assert_eq!(result, Ok(expr));
            Ok(())
        })
    }

    #[test]
    fn test_interpreting_assignment() {
        arbtest::builder().run(|u| {
            let name = u.arbitrary::<Identifier>()?;
            let value = u.arbitrary::<Primitive>()?;
            let expr = Expr::Let {
                annotation: (),
                name,
                value: Expr::Primitive {
                    annotation: (),
                    value: value.clone(),
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
                result,
                Ok(Expr::Primitive {
                    annotation: (),
                    value,
                }
                .into())
            );
            Ok(())
        })
    }

    #[test]
    fn test_interpreting_an_unknown_variable() {
        arbtest::builder().run(|u| {
            let name = u.arbitrary::<Identifier>()?;
            let expr = Expr::Identifier {
                annotation: (),
                name,
            };
            let result = interpret(expr.into());
            assert_eq!(
                result,
                Err(Error::UnknownVariable {
                    span: 0.into(), // this is wrong
                    name: name.to_string()
                })
            );
            Ok(())
        })
    }

    #[test]
    fn test_interpreting_addition() {
        test_interpreting_an_operation(Operation::Add, i64::checked_add)
    }

    #[test]
    fn test_interpreting_subtraction() {
        test_interpreting_an_operation(Operation::Subtract, i64::checked_sub)
    }

    #[test]
    fn test_interpreting_multiplication() {
        test_interpreting_an_operation(Operation::Multiply, i64::checked_mul)
    }

    fn test_interpreting_an_operation(
        operation: Operation,
        implementation: impl Fn(i64, i64) -> Option<i64>,
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
                            value: Primitive::Integer(left.into()),
                        }
                        .into(),
                        right: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Integer(right.into()),
                        }
                        .into(),
                    };
                    let result = interpret(expr.into());
                    assert_eq!(
                        result,
                        Ok(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Integer(expected.into()),
                        }
                        .into())
                    );
                    Ok(())
                }
            }
        })
    }

    #[test]
    fn test_interpreting_variable_use() {
        arbtest::builder().run(|u| {
            let name = u.arbitrary::<Identifier>()?;
            let variable = u.arbitrary::<i64>()?;
            let constant = u.arbitrary::<i64>()?;
            match variable.checked_add(constant) {
                None => Ok(()), // overflow or underflow
                Some(sum) => {
                    let expr = Rc::new(Expr::Let {
                        annotation: (),
                        name,
                        value: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Integer(variable.into()),
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
                                value: Primitive::Integer(constant.into()),
                            }
                            .into(),
                        }
                        .into(),
                    });
                    let result = interpret(expr.clone());
                    assert_eq!(
                        result,
                        Ok(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Integer(sum.into()),
                        }
                        .into())
                    );
                    Ok(())
                }
            }
        })
    }
}
