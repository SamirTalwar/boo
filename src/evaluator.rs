use std::rc::Rc;

use im::HashMap;

use crate::ast::*;
use crate::error::*;
use crate::identifier::*;
use crate::primitive::*;

pub fn evaluate<Annotation: Clone>(expr: Rc<Expr<Annotation>>) -> Result<Rc<Expr<Annotation>>> {
    evaluate_(expr, HashMap::new())
}

pub fn evaluate_<Annotation: Clone>(
    expr: Rc<Expr<Annotation>>,
    assignments: HashMap<Identifier, Rc<Expr<Annotation>>>,
) -> Result<Rc<Expr<Annotation>>> {
    match expr.as_ref() {
        Expr::Primitive { .. } => Ok(expr),
        Expr::Identifier {
            annotation: _,
            name,
        } => match assignments.get(name) {
            Some(value) => evaluate_(value.clone(), assignments),
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
        } => evaluate_(
            inner.clone(),
            assignments.update(name.clone(), value.clone()),
        ),
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
                let left_result = evaluate_(left.clone(), assignments.clone())?;
                let right_result = evaluate_(right.clone(), assignments.clone())?;
                evaluate_(
                    Expr::Infix {
                        annotation: annotation.clone(),
                        operation: *operation,
                        left: left_result,
                        right: right_result,
                    }
                    .into(),
                    assignments,
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
            let result = evaluate(expr.clone());
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
                name: name.clone(),
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
            let result = evaluate(expr.into());
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
                name: name.clone(),
            };
            let result = evaluate(expr.into());
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
        arbtest::builder().run(|u| {
            let left = u.arbitrary::<Integer>()?;
            let right = u.arbitrary::<Integer>()?;
            let expected = implementation(&left, &right);
            let expr = Expr::Infix {
                annotation: (),
                operation,
                left: Expr::Primitive {
                    annotation: (),
                    value: Primitive::Integer(left),
                }
                .into(),
                right: Expr::Primitive {
                    annotation: (),
                    value: Primitive::Integer(right),
                }
                .into(),
            };
            let result = evaluate(expr.into());
            assert_eq!(
                result,
                Ok(Expr::Primitive {
                    annotation: (),
                    value: Primitive::Integer(expected),
                }
                .into())
            );
            Ok(())
        })
    }

    #[test]
    fn test_interpreting_variable_use() {
        arbtest::builder().run(|u| {
            let name = u.arbitrary::<Identifier>()?;
            let variable = u.arbitrary::<Integer>()?;
            let constant = u.arbitrary::<Integer>()?;
            let sum = &variable + &constant;
            let expr = Rc::new(Expr::Let {
                annotation: (),
                name: name.clone(),
                value: Expr::Primitive {
                    annotation: (),
                    value: Primitive::Integer(variable),
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
                        value: Primitive::Integer(constant),
                    }
                    .into(),
                }
                .into(),
            });
            let result = evaluate(expr);
            assert_eq!(
                result,
                Ok(Expr::Primitive {
                    annotation: (),
                    value: Primitive::Integer(sum),
                }
                .into())
            );
            Ok(())
        })
    }
}
