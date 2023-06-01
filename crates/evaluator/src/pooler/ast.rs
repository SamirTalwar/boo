use boo_core::ast::Expression;
use boo_core::span::Spanned;

use super::pool::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(pub PoolRef<Spanned<Expression<Expr>>>);

impl Copy for Expr {}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub type ExprPool = Pool<Spanned<Expression<Expr>>>;
