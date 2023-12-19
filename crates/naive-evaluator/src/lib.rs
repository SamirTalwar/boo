//! Evaluates a core AST as simply as possible.
//!
//! This evaluator is not used by the interpreter. It is meant as an
//! implementation that is "so simple that there are obviously no deficiencies"
//! (to quote Tony Hoare). We then use it as a reference implementation to
//! validate that the real evaluator does the right thing when presented with an
//! arbitrary program.

use std::rc::Rc;

use im::HashSet;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::evaluation::*;
use boo_core::expr::Expr;
use boo_core::identifier::*;
use boo_core::native::*;
use boo_core::primitive::*;

/// Evaluates an AST as simply as possible.
pub struct NaiveEvaluator {
    bindings: Vec<(Identifier, Expr)>,
}

impl NaiveEvaluator {
    pub fn new() -> Self {
        Self { bindings: vec![] }
    }
}

impl Default for NaiveEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for NaiveEvaluator {
    fn bind(&mut self, identifier: Identifier, expr: Expr) -> Result<()> {
        self.bindings.push((identifier, expr));
        Ok(())
    }

    fn evaluate(&self, expr: Expr) -> Result<Evaluated> {
        let mut prepared = expr;
        for (identifier, value) in self.bindings.iter().rev() {
            prepared = Expr::new(
                None,
                Expression::Assign(Assign {
                    name: identifier.clone(),
                    value: value.clone(),
                    inner: prepared,
                }),
            );
        }
        evaluate(prepared)
    }
}

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
            match evaluate((*self.value).clone())? {
                Evaluated::Primitive(primitive) => Ok(primitive),
                Evaluated::Function(_) => Err(Error::TypeError),
            }
        } else {
            self.rest.lookup_value(identifier)
        }
    }
}

fn evaluate(expr: Expr) -> Result<Evaluated> {
    let mut progress = expr;
    loop {
        match step(progress)? {
            Progress::Next(next) => {
                progress = next;
            }
            Progress::Complete(complete) => {
                return match *complete.expression {
                    Expression::Primitive(primitive) => Ok(Evaluated::Primitive(primitive)),
                    Expression::Function(function) => Ok(Evaluated::Function(function)),
                    _ => unreachable!("Evaluated to a non-final expression."),
                };
            }
        }
    }
}

fn step(expr: Expr) -> Result<Progress<Expr>> {
    match *expr.expression {
        expression @ Expression::Primitive(_) | expression @ Expression::Function(_) => {
            Ok(Progress::Complete(Expr::new(expr.span, expression)))
        }
        Expression::Native(Native { implementation, .. }) => implementation(&EmptyContext {})
            .map(|x| Progress::Complete(Expr::new(expr.span, Expression::Primitive(x)))),
        Expression::Identifier(name) => Err(Error::UnknownVariable {
            span: expr.span,
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
        Expression::Match(Match {
            value,
            mut patterns,
        }) => {
            let PatternMatch { pattern, result } = patterns
                .pop_front()
                .ok_or(Error::MatchWithoutBaseCase { span: expr.span })?;
            match pattern {
                Pattern::Anything => Ok(Progress::Next(result)),
                _ => match step(value)? {
                    Progress::Next(value_next) => Ok(Progress::Next(Expr::new(
                        expr.span,
                        Expression::Match(Match {
                            value: value_next,
                            patterns,
                        }),
                    ))),
                    Progress::Complete(value_complete) => match pattern {
                        Pattern::Anything => unreachable!("Case should be handled already."),
                        Pattern::Primitive(expected) => match *value_complete.expression {
                            Expression::Primitive(actual) if actual == expected => {
                                Ok(Progress::Next(result))
                            }
                            // if not matched, try again, having discarded the first pattern
                            _ => Ok(Progress::Next(Expr::new(
                                expr.span,
                                Expression::Match(Match {
                                    value: value_complete,
                                    patterns,
                                }),
                            ))),
                        },
                    },
                },
            }
        }
        Expression::Apply(Apply { function, argument }) => {
            let function_result = step(function)?;
            match function_result {
                Progress::Next(function_next) => Ok(Progress::Next(Expr::new(
                    expr.span,
                    Expression::Apply(Apply {
                        function: function_next,
                        argument,
                    }),
                ))),
                Progress::Complete(function_complete) => match *function_complete.expression {
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
                    _ => Err(Error::InvalidFunctionApplication { span: expr.span }),
                },
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
    match *expr.expression {
        expression @ Expression::Primitive(_) => Expr::new(expr.span, expression),
        Expression::Native(Native {
            unique_name,
            implementation,
        }) => Expr::new(
            expr.span,
            Expression::Native(Native {
                unique_name,
                implementation: Rc::new(move |context| {
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
        expression @ Expression::Identifier(_) => Expr::new(expr.span, expression),
        Expression::Assign(Assign { name, value, inner }) if name != *substitution.name => {
            Expr::new(
                expr.span,
                Expression::Assign(Assign {
                    name: name.clone(),
                    value: substitute(substitution.clone(), value, bound.clone()),
                    inner: substitute(substitution, inner, bound.update(name)),
                }),
            )
        }
        expression @ Expression::Assign(_) => Expr::new(expr.span, expression),
        Expression::Function(Function { parameter, body }) if parameter != *substitution.name => {
            Expr::new(
                expr.span,
                Expression::Function(Function {
                    parameter: parameter.clone(),
                    body: substitute(substitution, body, bound.update(parameter)),
                }),
            )
        }
        expression @ Expression::Function(_) => Expr::new(expr.span, expression),
        Expression::Match(Match { value, patterns }) => Expr::new(
            expr.span,
            Expression::Match(Match {
                value: substitute(substitution.clone(), value, bound.clone()),
                patterns: patterns
                    .into_iter()
                    .map(|PatternMatch { pattern, result }| PatternMatch {
                        pattern,
                        result: substitute(substitution.clone(), result, bound.clone()),
                    })
                    .collect(),
            }),
        ),
        Expression::Apply(Apply { function, argument }) => Expr::new(
            expr.span,
            Expression::Apply(Apply {
                function: substitute(substitution.clone(), function, bound.clone()),
                argument: substitute(substitution, argument, bound),
            }),
        ),
    }
}

fn avoid_alpha_capture(expr: Expr, bound: HashSet<Identifier>) -> Expr {
    Expr::new(
        expr.span,
        match *expr.expression {
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
            Expression::Match(Match { value, patterns }) => Expression::Match(Match {
                value: avoid_alpha_capture(value, bound.clone()),
                patterns: patterns
                    .into_iter()
                    .map(|PatternMatch { pattern, result }| PatternMatch {
                        pattern,
                        result: avoid_alpha_capture(result, bound.clone()),
                    })
                    .collect(),
            }),
            Expression::Apply(Apply { function, argument }) => Expression::Apply(Apply {
                function: avoid_alpha_capture(function, bound.clone()),
                argument: avoid_alpha_capture(argument, bound),
            }),
        },
    )
}
