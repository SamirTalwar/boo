//! Evaluates a parsed AST as simply as possible.
//!
//! This evaluator is not used by the interpreter. It is meant as an
//! implementation that is "so simple that there are obviously no deficiencies"
//! (to quote Tony Hoare). We then use it as a reference implementation to
//! validate that the real evaluator does the right thing when presented with an
//! arbitrary program.

use std::rc::Rc;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::identifier::*;
use boo_core::operation::*;
use boo_core::primitive::*;
use boo_parser::Expr;

/// Evaluate a parsed AST as simply as possible.
pub fn naively_evaluate(expr: Expr) -> Result<Expr> {
    let span = expr.annotation();
    match expr.expression() {
        Expression::Primitive(value) => Ok(Expr::new(span, Expression::Primitive(value))),
        Expression::Identifier(name) => Err(Error::UnknownVariable {
            span,
            name: name.to_string(),
        }),
        Expression::Assign(Assign { name, value, inner }) => {
            let substituted_inner = substitute(
                Substitution {
                    name: name.into(),
                    value: value.into(),
                },
                inner,
            );
            naively_evaluate(substituted_inner)
        }
        Expression::Function(function) => Ok(Expr::new(span, Expression::Function(function))),
        Expression::Apply(Apply { function, argument }) => {
            let function_result = naively_evaluate(function)?;
            match function_result.expression() {
                Expression::Function(Function { parameter, body }) => {
                    let substituted_body = substitute(
                        Substitution {
                            name: parameter.into(),
                            value: argument.into(),
                        },
                        body,
                    );
                    naively_evaluate(substituted_body)
                }
                _ => Err(Error::InvalidFunctionApplication { span }),
            }
        }
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }) => {
            let left_result = naively_evaluate(left)?;
            let right_result = naively_evaluate(right)?;
            match (left_result.expression(), right_result.expression()) {
                (
                    Expression::Primitive(Primitive::Integer(left)),
                    Expression::Primitive(Primitive::Integer(right)),
                ) => Ok(Expr::new_unannotated(Expression::Primitive(
                    match operation {
                        Operation::Add => Primitive::Integer(left + right),
                        Operation::Subtract => Primitive::Integer(left - right),
                        Operation::Multiply => Primitive::Integer(left * right),
                    },
                ))),
                (left_result, right_result) => panic!(
                    "evaluate_infix branch is not implemented for:\n({}) {} ({})",
                    left_result, operation, right_result
                ),
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Substitution {
    name: Rc<Identifier>,
    value: Rc<Expr>,
}

fn substitute(substitution: Substitution, expr: Expr) -> Expr {
    let span = expr.annotation();
    match expr.expression() {
        expression @ Expression::Primitive(_) => Expr::new(span, expression),
        Expression::Identifier(name) if name == *substitution.name => (*substitution.value).clone(),
        expression @ Expression::Identifier(_) => Expr::new(span, expression),
        Expression::Assign(Assign { name, value, inner }) if name != *substitution.name => {
            Expr::new(
                span,
                Expression::Assign(Assign {
                    name,
                    value: substitute(substitution.clone(), value),
                    inner: substitute(substitution, inner),
                }),
            )
        }
        expression @ Expression::Assign(_) => Expr::new(span, expression),
        Expression::Function(Function { parameter, body }) if parameter != *substitution.name => {
            Expr::new(
                span,
                Expression::Function(Function {
                    parameter,
                    body: substitute(substitution, body),
                }),
            )
        }
        expression @ Expression::Function(_) => Expr::new(span, expression),
        Expression::Apply(Apply { function, argument }) => Expr::new(
            span,
            Expression::Apply(Apply {
                function: substitute(substitution.clone(), function),
                argument: substitute(substitution, argument),
            }),
        ),
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }) => Expr::new(
            span,
            Expression::Infix(Infix {
                operation,
                left: substitute(substitution.clone(), left),
                right: substitute(substitution, right),
            }),
        ),
    }
}
