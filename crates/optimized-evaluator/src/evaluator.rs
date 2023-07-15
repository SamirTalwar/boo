//! Evaluates a [pooled `Expr`][super::pooler::ast::Expr].

use std::borrow::Cow;
use std::sync::Arc;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::evaluation::*;
use boo_core::identifier::*;
use boo_core::native::*;
use boo_core::primitive::*;
use boo_core::span::Span;

use crate::ast::{Expr, ExprPool};
use crate::structures::{Bindings, EvaluatedBinding, EvaluationProgress};

/// Evaluate a [pooled `Expr`][super::pooler::ast::Expr].
pub fn evaluate(pool: &ExprPool, root: Expr) -> Result<Evaluated> {
    Evaluator {
        pool,
        bindings: Bindings::new(),
    }
    .evaluate(root)
    .map(|progress| progress.finish(pool))
}

/// An expression pool together with the current bound context, which can
/// evaluate a given expression reference from the pool.
struct Evaluator<'a> {
    pool: &'a ExprPool,
    bindings: Bindings<'a>,
}

impl<'a> Evaluator<'a> {
    /// Evaluates an expression from a pool in a given scope.
    ///
    /// The bindings are modified by assignment, accessed when evaluating an
    /// identifier, and captured by closures when a function is evaluated.
    pub fn evaluate(&self, expr_ref: Expr) -> Result<EvaluationProgress<'a>> {
        let expr = expr_ref.read_from(self.pool);
        match &expr.expression {
            Expression::Primitive(value) => Ok(EvaluationProgress::Primitive(Cow::Borrowed(value))),
            Expression::Native(Native { implementation, .. }) => {
                implementation(self).map(|value| EvaluationProgress::Primitive(Cow::Owned(value)))
            }
            Expression::Identifier(name) => self.resolve(name, expr.span),
            Expression::Assign(Assign {
                name,
                value: value_ref,
                inner: inner_ref,
            }) => self.with(name, *value_ref).evaluate(*inner_ref),
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
                            parameter,
                            *argument_ref,
                            self.bindings.clone(),
                        ))
                        .evaluate(*body_ref),
                    _ => Err(Error::InvalidFunctionApplication { span: expr.span }),
                }
            }
        }
    }

    /// Resolves a given identifier by evaluating its binding.
    fn resolve(&self, identifier: &Identifier, span: Option<Span>) -> EvaluatedBinding<'a> {
        match self.bindings.clone().read(identifier) {
            Some(thunk) => {
                let result = thunk.resolve_by(move |(value_ref, thunk_bindings)| {
                    self.switch(thunk_bindings.clone()).evaluate(*value_ref)
                });
                Arc::try_unwrap(result).unwrap_or_else(|arc| (*arc).clone())
            }
            None => Err(Error::UnknownVariable {
                span,
                name: identifier.to_string(),
            }),
        }
    }

    fn with(&self, identifier: &'a Identifier, expression: Expr) -> Self {
        self.switch(
            self.bindings
                .with(identifier, expression, self.bindings.clone()),
        )
    }

    fn switch(&self, new_bindings: Bindings<'a>) -> Self {
        Self {
            pool: self.pool,
            bindings: new_bindings,
        }
    }
}

impl<'a> NativeContext for Evaluator<'a> {
    fn lookup_value(&self, identifier: &Identifier) -> Result<Primitive> {
        match self.resolve(identifier, None)?.finish(self.pool) {
            Evaluated::Primitive(primitive) => Ok(primitive),
            Evaluated::Function(_) => Err(Error::TypeError),
        }
    }
}
