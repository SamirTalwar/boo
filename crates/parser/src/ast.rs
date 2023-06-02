use boo_core::ast::{Expression, ExpressionWrapper};
use boo_core::span::{Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(Box<Spanned<Expression<Expr>>>);

impl ExpressionWrapper for Expr {
    type Annotation = Span;

    fn map<Next>(self, f: &mut impl FnMut(Self::Annotation, Expression<Next>) -> Next) -> Next {
        let mapped = self.0.value.map(f);
        f(self.0.span, mapped)
    }
}

impl Expr {
    pub fn new(span: Span, value: Expression<Expr>) -> Self {
        Self(Spanned { span, value }.into())
    }

    pub fn span(&self) -> Span {
        self.0.span
    }

    pub fn value(&self) -> &Expression<Expr> {
        &self.0.value
    }

    pub fn consume(self) -> Expression<Expr> {
        self.0.value
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
