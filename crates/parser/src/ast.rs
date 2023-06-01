use boo_core::ast::Expression;
use boo_core::span::Spanned;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(pub Box<Spanned<Expression<Expr>>>);

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
