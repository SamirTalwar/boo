//! A simple expression wrapper.

use super::*;

/// A simple wrapper arond [`Expression`], mostly for testing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(Box<Expression<Expr>>);

impl Expr {
    pub fn new(expression: Expression<Expr>) -> Self {
        Self(Box::new(expression))
    }
}

impl ExpressionWrapper for Expr {
    type Annotation = ();

    fn new(_: (), expression: Expression<Expr>) -> Self {
        Self::new(expression)
    }

    fn new_unannotated(expression: Expression<Self>) -> Self {
        Self::new(expression)
    }

    fn annotation(&self) -> Self::Annotation {}

    fn expression(self) -> Expression<Self> {
        *self.0
    }
}

impl From<Expression<Expr>> for Expr {
    fn from(value: Expression<Expr>) -> Self {
        Expr::new(value)
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
