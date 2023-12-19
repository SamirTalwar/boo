//! Evaluates a [pooled `Expr`][super::pooler::ast::Expr].

use std::borrow::Cow;
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

use crate::ast::{Expr, ExprPool};
use crate::pooler::add_expr;
use crate::structures::Binding;
use crate::structures::{Bindings, EvaluatedBinding, EvaluationProgress};

/// An expression pool together with the current bound context, which can
/// evaluate a given expression reference from the pool.
pub struct OptimizedEvaluator {
    pool: ExprPool,
    bindings: collections::HashMap<Identifier, Expr>,
}

impl OptimizedEvaluator {
    pub fn new() -> Self {
        Self {
            pool: ExprPool::new(),
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
                    bindings.with(Cow::Borrowed(identifier), *pool_ref, Bindings::new())
                });
        let inner = InnerEvaluator {
            pool: &pool,
            bindings,
        };
        inner.evaluate(root).map(|progress| progress.finish(&pool))
    }
}

struct InnerEvaluator<'a> {
    pool: &'a ExprPool,
    bindings: Bindings<'a>,
}

impl<'a> InnerEvaluator<'a> {
    /// Evaluates an expression from a pool in a given scope.
    ///
    /// The bindings are modified by assignment, accessed when evaluating an
    /// identifier, and captured by closures when a function is evaluated.
    fn evaluate(&self, expr_ref: Expr) -> Result<EvaluationProgress<'a>> {
        let expr = expr_ref.read_from(self.pool);
        match &expr.expression {
            Expression::Primitive(value) => Ok(EvaluationProgress::Primitive(Cow::Borrowed(value))),
            Expression::Native(Native { implementation, .. }) => {
                implementation(self).map(|value| EvaluationProgress::Primitive(Cow::Owned(value)))
            }
            Expression::Identifier(name) => self.resolve(name, expr.span),
            Expression::Function(function) => {
                Ok(EvaluationProgress::Closure(function, self.bindings.clone()))
            }
            Expression::Apply(Apply {
                function: function_ref,
                argument: argument_ref,
            }) => {
                let function_result = self.evaluate(*function_ref)?;
                match function_result {
                    EvaluationProgress::Closure(
                        Function {
                            parameter,
                            body: body_ref,
                        },
                        function_bindings,
                    ) => self
                        // the body is executed in the context of the function,
                        // but the argument must be evaluated in the outer context
                        .switch(function_bindings.with(
                            Cow::Borrowed(parameter),
                            *argument_ref,
                            self.bindings.clone(),
                        ))
                        .evaluate(*body_ref),
                    _ => Err(Error::InvalidFunctionApplication { span: expr.span }),
                }
            }
            Expression::Assign(Assign {
                name,
                value: value_ref,
                inner: inner_ref,
            }) => self.with(name, *value_ref).evaluate(*inner_ref),
            Expression::Match(Match {
                value: value_ref,
                patterns,
            }) => {
                // Ensure we only evaluate the value once.
                let mut value = Binding::unresolved((*value_ref, self.bindings.clone()));
                for PatternMatch {
                    pattern,
                    result: result_ref,
                } in patterns
                {
                    match pattern {
                        Pattern::Anything => {
                            return self.evaluate(*result_ref);
                        }
                        Pattern::Primitive(expected) => {
                            let resolved_value = self.resolve_binding(&mut value)?;
                            match resolved_value {
                                EvaluationProgress::Primitive(actual)
                                    if actual.as_ref() == expected =>
                                {
                                    return self.evaluate(*result_ref);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::MatchWithoutBaseCase { span: expr.span })
            }
        }
    }

    /// Resolves a given identifier by evaluating it in the context of the bindings.
    fn resolve(&self, identifier: &Identifier, span: Option<Span>) -> EvaluatedBinding<'a> {
        match self.bindings.clone().read(identifier) {
            Some(binding) => self.resolve_binding(binding),
            None => Err(Error::UnknownVariable {
                span,
                name: identifier.to_string(),
            }),
        }
    }

    /// Resolves a given binding in context.
    fn resolve_binding(&self, binding: &mut Binding<'a>) -> EvaluatedBinding<'a> {
        let result = binding.resolve_by(move |(value_ref, thunk_bindings)| {
            self.switch(thunk_bindings.clone()).evaluate(*value_ref)
        });
        Arc::try_unwrap(result).unwrap_or_else(|arc| (*arc).clone())
    }

    fn with(&self, identifier: &'a Identifier, expression: Expr) -> Self {
        self.switch(self.bindings.with(
            Cow::Borrowed(identifier),
            expression,
            self.bindings.clone(),
        ))
    }

    fn switch(&self, new_bindings: Bindings<'a>) -> Self {
        Self {
            pool: self.pool,
            bindings: new_bindings,
        }
    }
}

impl<'a> NativeContext for InnerEvaluator<'a> {
    fn lookup_value(&self, identifier: &Identifier) -> Result<Primitive> {
        match self.resolve(identifier, None)?.finish(self.pool) {
            Evaluated::Primitive(primitive) => Ok(primitive),
            Evaluated::Function(_) => Err(Error::TypeError),
        }
    }
}
