//! Evaluates an expression recursively.

use std::sync::Arc;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::evaluation::*;
use boo_core::identifier::*;
use boo_core::native::*;
use boo_core::primitive::*;
use boo_core::span::Span;
use boo_core::span::Spanned;
use boo_evaluation_lazy::{Binding, Bindings, CompletedEvaluation, EvaluatedBinding};

pub struct RecursiveEvaluator<'a, Expr: Clone, Reader: ExpressionReader<Expr = Expr>> {
    reader: Reader,
    bindings: Bindings<'a, Expr>,
}

impl<'a, Expr: Clone, Reader: ExpressionReader<Expr = Expr>> RecursiveEvaluator<'a, Expr, Reader> {
    pub fn new(reader: Reader, bindings: Bindings<'a, Expr>) -> Self {
        Self { reader, bindings }
    }

    /// Evaluates an expression from a pool in a given scope.
    ///
    /// The bindings are modified by assignment, accessed when evaluating an
    /// identifier, and captured by closures when a function is evaluated.
    pub fn evaluate(&self, expr: Expr) -> Result<CompletedEvaluation<'a, Expr>> {
        let Spanned {
            span,
            value: expression,
        } = self.reader.read(expr);
        match expression.as_ref() {
            Expression::Primitive(value) => Ok(CompletedEvaluation::Primitive(value.clone())),
            Expression::Native(Native { implementation, .. }) => {
                implementation(self).map(CompletedEvaluation::Primitive)
            }
            Expression::Identifier(name) => self.resolve(name, span),
            Expression::Function(Function { parameter, body }) => {
                Ok(CompletedEvaluation::Closure {
                    parameter: parameter.clone(),
                    body: body.clone(),
                    bindings: self.bindings.clone(),
                })
            }
            Expression::Apply(Apply { function, argument }) => {
                let function_result = self.evaluate(function.clone())?;
                match function_result {
                    CompletedEvaluation::Closure {
                        parameter,
                        body,
                        bindings: function_bindings,
                    } => self
                        // the body is executed in the context of the function,
                        // but the argument must be evaluated in the outer context
                        .switch(function_bindings.with(
                            parameter.clone(),
                            argument.clone(),
                            self.bindings.clone(),
                        ))
                        .evaluate(body),
                    _ => Err(Error::InvalidFunctionApplication { span }),
                }
            }
            Expression::Assign(Assign { name, value, inner }) => self
                .switch(
                    self.bindings
                        .with(name.clone(), value.clone(), self.bindings.clone()),
                )
                .evaluate(inner.clone()),
            Expression::Match(Match { value, patterns }) => {
                // Ensure we only evaluate the value once.
                let mut value = Binding::unresolved((value.clone(), self.bindings.clone()));
                for PatternMatch { pattern, result } in patterns {
                    match pattern {
                        Pattern::Anything => {
                            return self.evaluate(result.clone());
                        }
                        Pattern::Primitive(expected) => {
                            let resolved_value = self.resolve_binding(&mut value)?;
                            match resolved_value {
                                CompletedEvaluation::Primitive(actual) if actual == *expected => {
                                    return self.evaluate(result.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::MatchWithoutBaseCase { span })
            }
            Expression::Typed(Typed { expression, typ: _ }) => self.evaluate(expression.clone()),
        }
    }

    /// Resolves a given identifier by evaluating it in the context of the bindings.
    fn resolve(&self, identifier: &Identifier, span: Option<Span>) -> EvaluatedBinding<'a, Expr> {
        match self.bindings.clone().read(identifier) {
            Some(binding) => self.resolve_binding(binding),
            None => Err(Error::UnknownVariable {
                span,
                name: identifier.to_string(),
            }),
        }
    }

    /// Resolves a given binding in context.
    fn resolve_binding(&self, binding: &mut Binding<'a, Expr>) -> EvaluatedBinding<'a, Expr> {
        let result = binding.resolve_by(move |(value, thunk_bindings)| {
            self.switch(thunk_bindings.clone()).evaluate(value.clone())
        });
        Arc::try_unwrap(result).unwrap_or_else(|arc| (*arc).clone())
    }

    fn switch(&self, new_bindings: Bindings<'a, Expr>) -> Self {
        Self {
            reader: self.reader,
            bindings: new_bindings,
        }
    }
}

impl<'a, Expr: Clone, Reader: ExpressionReader<Expr = Expr>> NativeContext
    for RecursiveEvaluator<'a, Expr, Reader>
{
    fn lookup_value(&self, identifier: &Identifier) -> Result<Primitive> {
        match self.resolve(identifier, None)?.finish(self.reader) {
            Evaluated::Primitive(primitive) => Ok(primitive),
            Evaluated::Function(_) => Err(Error::InvalidPrimitive { span: None }),
        }
    }
}
