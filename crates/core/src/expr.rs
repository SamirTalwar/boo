//! The core Boo AST, represented as a wrapped [`Expression`].

pub use crate::ast::*;
use crate::evaluation::ExpressionReader;
use crate::span::*;

/// Wraps an expression with a span.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Expr(Spanned<Box<Expression<Expr>>>);

impl Expr {
    pub fn new(span: Option<Span>, expression: Expression<Self>) -> Self {
        Self(Spanned {
            span,
            value: expression.into(),
        })
    }

    pub fn expression(&self) -> &Expression<Expr> {
        self.0.value.as_ref()
    }

    pub fn take(self) -> Expression<Expr> {
        *self.0.value
    }

    pub fn span(&self) -> Option<Span> {
        self.0.span
    }
}

// We use this for testing, and the default implementation is a bit ugly.
impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Expr")
            .field("span", &self.0.span)
            .field("expression", self.0.value.as_ref())
            .finish()
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.value.fmt(f)
    }
}

#[derive(Clone, Copy)]
pub struct ExprReader;

impl ExpressionReader for ExprReader {
    type Expr = self::Expr;
    type Target = Box<Expression<Self::Expr>>;

    fn read(&self, expr: Self::Expr) -> Spanned<Self::Target> {
        expr.0
    }

    fn to_core(&self, expr: Self::Expr) -> Expr {
        expr
    }
}
