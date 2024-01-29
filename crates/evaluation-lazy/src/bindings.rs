//! Core data structures used by the evaluator.

use im::HashMap;

use boo_core::error::Result;
use boo_core::identifier::Identifier;

use crate::completed::CompletedEvaluation;
use crate::thunk::Thunk;

pub type UnevaluatedBinding<'a, Expr> = (Expr, Bindings<'a, Expr>);
pub type EvaluatedBinding<'a, Expr> = Result<CompletedEvaluation<'a, Expr>>;
pub type Binding<'a, Expr> = Thunk<UnevaluatedBinding<'a, Expr>, EvaluatedBinding<'a, Expr>>;

/// The set of bindings in a given scope.
///
/// The variables bound in a specific scope are a mapping from an identifier to
/// the underlying expression. This expression is evaluated lazily, but only
/// once, using [`Thunk`].
#[derive(Debug, Clone)]
pub struct Bindings<'a, Expr: Clone>(HashMap<Identifier, Binding<'a, Expr>>);

impl<'a, Expr: Clone> Bindings<'a, Expr> {
    /// Constructs an empty set of bindings.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn read(
        &mut self,
        identifier: &Identifier,
    ) -> Option<&mut Thunk<UnevaluatedBinding<'a, Expr>, EvaluatedBinding<'a, Expr>>> {
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

impl<'a, Expr: Clone> Default for Bindings<'a, Expr> {
    fn default() -> Self {
        Self::new()
    }
}
