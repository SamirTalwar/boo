//! Evaluates a [pooled `Expr`][super::pooler::ast::Expr].

use std::collections;
use std::sync::Arc;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::evaluation::*;
use boo_core::expr::Expr as CoreExpr;
use boo_core::identifier::*;
use boo_core::native::*;
use boo_core::primitive::*;
use boo_core::span::Span;
use boo_core::span::Spanned;
use boo_evaluation_lazy::{Binding, Bindings, CompletedEvaluation, EvaluatedBinding};

use crate::ast;
use crate::pooler::add_expr;

/// An expression pool together with the current bound context, which can
/// evaluate a given expression reference from the pool.
pub struct OptimizedEvaluator {
    pool: ast::ExprPool,
    bindings: collections::HashMap<Identifier, ast::Expr>,
}

impl OptimizedEvaluator {
    pub fn new() -> Self {
        Self {
            pool: ast::ExprPool::new(),
            bindings: collections::HashMap::new(),
        }
    }
}

impl Default for OptimizedEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for OptimizedEvaluator {
    fn bind(&mut self, identifier: Identifier, expr: CoreExpr) -> Result<()> {
        let pool_ref = add_expr(&mut self.pool, expr);
        self.bindings.insert(identifier, pool_ref);
        Ok(())
    }

    fn evaluate(&self, expr: CoreExpr) -> Result<Evaluated> {
        let mut pool = self.pool.clone();
        let root = add_expr(&mut pool, expr);
        let bindings =
            self.bindings
                .iter()
                .fold(Bindings::new(), |bindings, (identifier, pool_ref)| {
                    bindings.with(identifier.clone(), *pool_ref, Bindings::new())
                });
        let inner = InnerEvaluator {
            reader: &pool,
            bindings,
        };
        inner.evaluate(root).map(|progress| progress.finish(&pool))
    }
}

struct InnerEvaluator<'a, Expr: Clone, Reader: ExpressionReader<Expr = Expr>> {
    reader: Reader,
    bindings: Bindings<'a, Expr>,
}

impl<'a, Expr: Clone, Reader: ExpressionReader<Expr = Expr>> InnerEvaluator<'a, Expr, Reader> {
    /// Evaluates an expression from a pool in a given scope.
    ///
    /// The bindings are modified by assignment, accessed when evaluating an
    /// identifier, and captured by closures when a function is evaluated.
    fn evaluate(&self, expr: Expr) -> Result<CompletedEvaluation<'a, Expr>> {
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
    for InnerEvaluator<'a, Expr, Reader>
{
    fn lookup_value(&self, identifier: &Identifier) -> Result<Primitive> {
        match self.resolve(identifier, None)?.finish(self.reader) {
            Evaluated::Primitive(primitive) => Ok(primitive),
            Evaluated::Function(_) => Err(Error::InvalidPrimitive { span: None }),
        }
    }
}
