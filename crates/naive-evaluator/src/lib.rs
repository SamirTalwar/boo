//! Evaluates a parsed AST as simply as possible.
//!
//! This evaluator is not used by the interpreter. It is meant as an
//! implementation that is "so simple that there are obviously no deficiencies"
//! (to quote Tony Hoare). We then use it as a reference implementation to
//! validate that the real evaluator does the right thing when presented with an
//! arbitrary program.

use std::rc::Rc;
use std::sync::Arc;

use im::HashSet;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::expr::Expr;
use boo_core::identifier::*;
use boo_core::native::*;
use boo_core::primitive::*;

enum Progress<T> {
    Next(T),
    Complete(T),
}

struct EmptyContext {}

impl NativeContext for EmptyContext {
    fn lookup_value(&self, identifier: &Identifier) -> Result<Primitive> {
        Err(Error::UnknownVariable {
            span: None,
            name: identifier.to_string(),
        })
    }
}

struct AdditionalContext<'a> {
    name: Rc<Identifier>,
    value: Rc<Expr>,
    rest: &'a dyn NativeContext,
}

impl<'a> NativeContext for AdditionalContext<'a> {
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
pub fn naively_evaluate(expr: Expr) -> Result<Expr> {
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

fn step(expr: Expr) -> Result<Progress<Expr>> {
    let span = expr.annotation();
    match expr.expression() {
        expression @ Expression::Primitive(_) | expression @ Expression::Function(_) => {
            Ok(Progress::Complete(Expr::new(span, expression)))
        }
        Expression::Native(Native { implementation, .. }) => implementation(&EmptyContext {})
            .map(|x| Progress::Complete(Expr::new(span, Expression::Primitive(x)))),
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
                HashSet::new(),
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
                        HashSet::new(),
                    );
                    Ok(Progress::Next(substituted_body))
                }
                _ => Err(Error::InvalidFunctionApplication { span }),
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Substitution {
    name: Rc<Identifier>,
    value: Rc<Expr>,
}

fn substitute(substitution: Substitution, expr: Expr, bound: HashSet<Identifier>) -> Expr {
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
                    implementation(&AdditionalContext {
                        name: substitution.name.clone(),
                        value: substitution.value.clone(),
                        rest: context,
                    })
                }),
            }),
        ),
        Expression::Identifier(name) if name == *substitution.name => {
            avoid_alpha_capture((*substitution.value).clone(), bound)
        }
        expression @ Expression::Identifier(_) => Expr::new(annotation, expression),
        Expression::Assign(Assign { name, value, inner }) if name != *substitution.name => {
            Expr::new(
                annotation,
                Expression::Assign(Assign {
                    name: name.clone(),
                    value: substitute(substitution.clone(), value, bound.clone()),
                    inner: substitute(substitution, inner, bound.update(name)),
                }),
            )
        }
        expression @ Expression::Assign(_) => Expr::new(annotation, expression),
        Expression::Function(Function { parameter, body }) if parameter != *substitution.name => {
            Expr::new(
                annotation,
                Expression::Function(Function {
                    parameter: parameter.clone(),
                    body: substitute(substitution, body, bound.update(parameter)),
                }),
            )
        }
        expression @ Expression::Function(_) => Expr::new(annotation, expression),
        Expression::Apply(Apply { function, argument }) => Expr::new(
            annotation,
            Expression::Apply(Apply {
                function: substitute(substitution.clone(), function, bound.clone()),
                argument: substitute(substitution, argument, bound),
            }),
        ),
    }
}

fn avoid_alpha_capture(expr: Expr, bound: HashSet<Identifier>) -> Expr {
    let annotation = expr.annotation();
    Expr::new(
        annotation,
        match expr.expression() {
            expression @ Expression::Primitive(_) | expression @ Expression::Native(_) => {
                expression
            }
            Expression::Identifier(identifier) if bound.contains(&identifier) => {
                let original = Rc::new(identifier);
                let new_identifier = (1u32..)
                    .map(|suffix| Identifier::AvoidingCapture {
                        original: original.clone(),
                        suffix,
                    })
                    .find(|i| !bound.contains(i))
                    .unwrap();
                Expression::Identifier(new_identifier)
            }
            Expression::Identifier(identifier) => Expression::Identifier(identifier),
            Expression::Assign(Assign { name, value, inner }) => Expression::Assign(Assign {
                name,
                value: avoid_alpha_capture(value, bound.clone()),
                inner: avoid_alpha_capture(inner, bound),
            }),
            Expression::Function(Function { parameter, body }) => Expression::Function(Function {
                parameter,
                body: avoid_alpha_capture(body, bound),
            }),
            Expression::Apply(Apply { function, argument }) => Expression::Apply(Apply {
                function: avoid_alpha_capture(function, bound.clone()),
                argument: avoid_alpha_capture(argument, bound),
            }),
        },
    )
}
