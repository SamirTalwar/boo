//! The core Boo AST, represented as a wrapped [`Expression`].

pub use crate::ast::*;
use crate::span::*;

/// Wraps an expression with a span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(Box<Spanned<Expression<Expr>>>);

impl HasSpan for Expr {
    fn span(&self) -> crate::span::Span {
        self.0.span
    }
}

impl ExpressionWrapper for Expr {
    type Annotation = Span;

    fn new(span: Self::Annotation, value: Expression<Self>) -> Self {
        Expr(Box::new(Spanned { span, value }))
    }

    fn new_unannotated(expression: Expression<Self>) -> Self {
        Self::new(0.into(), expression)
    }

    fn annotation(&self) -> Self::Annotation {
        self.0.span
    }

    fn expression(self) -> Expression<Self> {
        self.0.value
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.value.fmt(f)
    }
}
