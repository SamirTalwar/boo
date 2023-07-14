//! The core Boo AST, represented as a wrapped [`Expression`].

pub use crate::ast::*;
use crate::span::*;

/// Wraps an expression with a span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    span: Option<Span>,
    expression: Box<Expression<Expr>>,
}

impl ExpressionWrapper for Expr {
    type Annotation = Option<Span>;

    fn new(span: Self::Annotation, expression: Expression<Self>) -> Self {
        Self {
            span,
            expression: expression.into(),
        }
    }

    fn new_unannotated(expression: Expression<Self>) -> Self {
        Self::new(None, expression)
    }

    fn annotation(&self) -> Self::Annotation {
        self.span
    }

    fn expression(self) -> Expression<Self> {
        *self.expression
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.expression.fmt(f)
    }
}
