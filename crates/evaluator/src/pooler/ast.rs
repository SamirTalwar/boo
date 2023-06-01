use boo_core::ast::Expression;
use boo_core::span::{Span, Spanned};

use super::pool::*;

type Inner = Spanned<Expression<Expr>>;

pub type ExprPool = Pool<Inner>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(PoolRef<Inner>);

impl Expr {
    pub fn insert(pool: &mut ExprPool, span: Span, value: Expression<Expr>) -> Self {
        Self(pool.add(Spanned { span, value }))
    }

    pub fn from_root(pool: &ExprPool) -> Self {
        Self(pool.root())
    }

    pub fn read_from<'a>(&self, pool: &'a ExprPool) -> &'a Inner {
        pool.get(self.0)
    }
}

impl Copy for Expr {}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
