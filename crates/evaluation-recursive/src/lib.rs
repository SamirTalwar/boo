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

pub struct RecursiveEvaluator<Expr: Clone, Reader: ExpressionReader<Expr = Expr>> {
    reader: Reader,
    bindings: Bindings<Expr>,
}

impl<Expr: Clone, Reader: ExpressionReader<Expr = Expr>> RecursiveEvaluator<Expr, Reader> {
    pub fn new(reader: Reader, bindings: Bindings<Expr>) -> Self {
        Self { reader, bindings }
    }
}

impl<Expr: Clone, Reader: ExpressionReader<Expr = Expr>> Evaluator<Expr>
    for RecursiveEvaluator<Expr, Reader>
{
    fn bind(&mut self, identifier: Identifier, expr: Expr) -> Result<()> {
        self.bindings = self.bindings.with(identifier, expr, Bindings::new());
        Ok(())
    }

    /// Evaluates an expression from a pool in a given scope.
    ///
    /// The bindings are modified by assignment, accessed when evaluating an
    /// identifier, and captured by closures when a function is evaluated.
    fn evaluate(&self, expr: Expr) -> Result<Evaluated<Expr>> {
        self.evaluate_inner(expr)
            .map(|completed| completed.finish())
    }
}

impl<Expr: Clone, Reader: ExpressionReader<Expr = Expr>> RecursiveEvaluator<Expr, Reader> {
    fn evaluate_inner(&self, expr: Expr) -> Result<CompletedEvaluation<Expr>> {
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
                let function_result = self.evaluate_inner(function.clone())?;
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
                        .evaluate_inner(body),
                    _ => Err(Error::InvalidFunctionApplication { span }),
                }
            }
            Expression::Assign(Assign { name, value, inner }) => self
                .switch(
                    self.bindings
                        .with(name.clone(), value.clone(), self.bindings.clone()),
                )
                .evaluate_inner(inner.clone()),
            Expression::Match(Match { value, patterns }) => {
                // Ensure we only evaluate the value once.
                let mut value = Binding::unresolved((value.clone(), self.bindings.clone()));
                for PatternMatch { pattern, result } in patterns {
                    match pattern {
                        Pattern::Anything => {
                            return self.evaluate_inner(result.clone());
                        }
                        Pattern::Primitive(expected) => {
                            let resolved_value = self.resolve_binding(&mut value)?;
                            match resolved_value {
                                CompletedEvaluation::Primitive(actual) if actual == *expected => {
                                    return self.evaluate_inner(result.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::MatchWithoutBaseCase { span })
            }
            Expression::Typed(Typed { expression, typ: _ }) => {
                self.evaluate_inner(expression.clone())
            }
        }
    }

    /// Resolves a given identifier by evaluating it in the context of the bindings.
    fn resolve(&self, identifier: &Identifier, span: Option<Span>) -> EvaluatedBinding<Expr> {
        match self.bindings.clone().read(identifier) {
            Some(binding) => self.resolve_binding(binding),
            None => Err(Error::UnknownVariable {
                span,
                name: identifier.to_string(),
            }),
        }
    }

    /// Resolves a given binding in context.
    fn resolve_binding(&self, binding: &mut Binding<Expr>) -> EvaluatedBinding<Expr> {
        let result = binding.resolve_by(move |(value, thunk_bindings)| {
            self.switch(thunk_bindings.clone())
                .evaluate_inner(value.clone())
        });
        Arc::try_unwrap(result).unwrap_or_else(|arc| (*arc).clone())
    }

    fn switch(&self, new_bindings: Bindings<Expr>) -> Self {
        Self {
            reader: self.reader,
            bindings: new_bindings,
        }
    }
}

impl<Expr: Clone, Reader: ExpressionReader<Expr = Expr>> NativeContext
    for RecursiveEvaluator<Expr, Reader>
{
    fn lookup_value(&self, identifier: &Identifier) -> Result<Primitive> {
        match self.resolve(identifier, None)?.finish() {
            Evaluated::Primitive(primitive) => Ok(primitive),
            Evaluated::Function(_) => Err(Error::InvalidPrimitive { span: None }),
        }
    }
}
