use boo_core::ast::{Expression, ExpressionWrapper};
use boo_core::span::{Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(Box<Spanned<Expression<Expr>>>);

impl ExpressionWrapper for Expr {
    type Annotation = Span;

    fn new(span: Self::Annotation, value: Expression<Self>) -> Self {
        Self(Spanned { span, value }.into())
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
        self.0.fmt(f)
    }
}
