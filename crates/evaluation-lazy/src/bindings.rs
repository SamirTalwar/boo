//! Core data structures used by the evaluator.

use im::HashMap;

use boo_core::error::Result;
use boo_core::identifier::Identifier;

use crate::completed::CompletedEvaluation;
use crate::thunk::Thunk;

pub type UnevaluatedBinding<Expr> = (Expr, Bindings<Expr>);
pub type EvaluatedBinding<Expr> = Result<CompletedEvaluation<Expr>>;
pub type Binding<Expr> = Thunk<UnevaluatedBinding<Expr>, EvaluatedBinding<Expr>>;

/// The set of bindings in a given scope.
///
/// The variables bound in a specific scope are a mapping from an identifier to
/// the underlying expression. This expression is evaluated lazily, but only
/// once, using [`Thunk`].
#[derive(Debug, Clone)]
pub struct Bindings<Expr: Clone>(HashMap<Identifier, Binding<Expr>>);

impl<Expr: Clone> Bindings<Expr> {
    /// Constructs an empty set of bindings.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn read(
        &mut self,
        identifier: &Identifier,
    ) -> Option<&mut Thunk<UnevaluatedBinding<Expr>, EvaluatedBinding<Expr>>> {
        self.0.get_mut(identifier)
    }

    /// Adds a new binding to the set.
    pub fn with(
        &self,
        identifier: Identifier,
        expression: Expr,
        expression_bindings: Self,
    ) -> Self {
        Self(self.0.update(
            identifier,
            Thunk::unresolved((expression, expression_bindings)),
        ))
    }
}

impl<Expr: Clone> Default for Bindings<Expr> {
    fn default() -> Self {
        Self::new()
    }
}
