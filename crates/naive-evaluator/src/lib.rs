//! Evaluates a parsed AST as simply as possible.
//!
//! This evaluator is not used by the interpreter. It is meant as an
//! implementation that is "so simple that there are obviously no deficiencies"
//! (to quote Tony Hoare). We then use it as a reference implementation to
//! validate that the real evaluator does the right thing when presented with an
//! arbitrary program.

use std::rc::Rc;
use std::sync::Arc;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::identifier::*;
use boo_core::native::*;
use boo_core::operation::*;
use boo_core::primitive::*;
use boo_core::span::HasSpan;

enum Progress<T> {
    Next(T),
    Complete(T),
}

struct EmptyContext {}

impl NativeContext for EmptyContext {
    fn lookup_value(&self, identifier: &Identifier) -> Result<Primitive> {
        Err(Error::UnknownVariable {
            span: 0.into(),
            name: identifier.to_string(),
        })
    }
}

struct AdditionalContext<Expr> {
    name: Rc<Identifier>,
    value: Rc<Expr>,
    rest: Box<dyn NativeContext>,
}

impl<Expr> NativeContext for AdditionalContext<Expr>
where
    Expr: ExpressionWrapper + HasSpan + Clone + 'static,
{
    fn lookup_value(&self, identifier: &Identifier) -> Result<Primitive> {
        if identifier == self.name.as_ref() {
            match naively_evaluate((*self.value).clone())?.expression() {
                Expression::Primitive(primitive) => Ok(primitive),
                _ => Err(Error::TypeError),
            }
        } else {
            self.rest.lookup_value(identifier)
        }
    }
}

/// Evaluate a parsed AST as simply as possible.
pub fn naively_evaluate<Expr>(expr: Expr) -> Result<Expr>
where
    Expr: ExpressionWrapper + HasSpan + Clone + 'static,
{
    let mut progress = expr;
    loop {
        match step(progress)? {
            Progress::Next(next) => {
                progress = next;
            }
            Progress::Complete(complete) => {
                return Ok(complete);
            }
        }
    }
}

fn step<Expr>(expr: Expr) -> Result<Progress<Expr>>
where
    Expr: ExpressionWrapper + HasSpan + Clone + 'static,
{
    let annotation = expr.annotation();
    let span = expr.span();
    match expr.expression() {
        expression @ Expression::Primitive(_) | expression @ Expression::Function(_) => {
            Ok(Progress::Complete(Expr::new(annotation, expression)))
        }
        Expression::Native(Native { implementation, .. }) => {
            implementation(Box::new(EmptyContext {}))
                .map(|x| Progress::Complete(Expr::new(annotation, Expression::Primitive(x))))
        }
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
            Ok(Progress::Next(substituted_inner))
        }
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
                    Ok(Progress::Next(substituted_body))
                }
                _ => Err(Error::InvalidFunctionApplication { span }),
            }
        }
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }) => match step(left)? {
            Progress::Next(left_next) => Ok(Progress::Next(Expr::new(
                annotation,
                Expression::Infix(Infix {
                    operation,
                    left: left_next,
                    right,
                }),
            ))),
            Progress::Complete(left) => match step(right)? {
                Progress::Next(right_next) => Ok(Progress::Next(Expr::new(
                    annotation,
                    Expression::Infix(Infix {
                        operation,
                        left,
                        right: right_next,
                    }),
                ))),
                Progress::Complete(right) => match (left.expression(), right.expression()) {
                    (
                        Expression::Primitive(Primitive::Integer(left)),
                        Expression::Primitive(Primitive::Integer(right)),
                    ) => Ok(Progress::Next(Expr::new_unannotated(
                        Expression::Primitive(match operation {
                            Operation::Add => Primitive::Integer(left + right),
                            Operation::Subtract => Primitive::Integer(left - right),
                            Operation::Multiply => Primitive::Integer(left * right),
                        }),
                    ))),
                    _ => Err(Error::TypeError),
                },
            },
        },
    }
}

#[derive(Debug, Clone)]
struct Substitution<Expr: ExpressionWrapper + HasSpan> {
    name: Rc<Identifier>,
    value: Rc<Expr>,
}

fn substitute<Expr>(substitution: Substitution<Expr>, expr: Expr) -> Expr
where
    Expr: ExpressionWrapper + HasSpan + Clone + 'static,
{
    let annotation = expr.annotation();
    match expr.expression() {
        expression @ Expression::Primitive(_) => Expr::new(annotation, expression),
        Expression::Native(Native {
            unique_name,
            implementation,
        }) => Expr::new(
            annotation,
            Expression::Native(Native {
                unique_name,
                implementation: Arc::new(move |context| {
                    implementation(Box::new(AdditionalContext {
                        name: substitution.name.clone(),
                        value: substitution.value.clone(),
                        rest: context,
                    }))
                }),
            }),
        ),
        Expression::Identifier(name) if name == *substitution.name => (*substitution.value).clone(),
        expression @ Expression::Identifier(_) => Expr::new(annotation, expression),
        Expression::Assign(Assign { name, value, inner }) if name != *substitution.name => {
            Expr::new(
                annotation,
                Expression::Assign(Assign {
                    name,
                    value: substitute(substitution.clone(), value),
                    inner: substitute(substitution, inner),
                }),
            )
        }
        expression @ Expression::Assign(_) => Expr::new(annotation, expression),
        Expression::Function(Function { parameter, body }) if parameter != *substitution.name => {
            Expr::new(
                annotation,
                Expression::Function(Function {
                    parameter,
                    body: substitute(substitution, body),
                }),
            )
        }
        expression @ Expression::Function(_) => Expr::new(annotation, expression),
        Expression::Apply(Apply { function, argument }) => Expr::new(
            annotation,
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
            annotation,
            Expression::Infix(Infix {
                operation,
                left: substitute(substitution.clone(), left),
                right: substitute(substitution, right),
            }),
        ),
    }
}
