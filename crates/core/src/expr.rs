//! The core Boo AST, represented as a wrapped [`Expression`].

pub use crate::ast::*;
use crate::span::*;

/// Wraps an expression with a span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    pub span: Option<Span>,
    pub expression: Box<Expression<Expr>>,
}

impl Expr {
    pub fn new(span: Option<Span>, expression: Expression<Self>) -> Self {
        Self {
            span,
            expression: expression.into(),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.expression.fmt(f)
    }
}
