//! An AST based on a [`Pool`].

use boo_core::ast::Expression;
use boo_core::span::Span;

use super::pool::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Inner {
    pub span: Option<Span>,
    pub expression: Expression<Expr>,
}

/// An expression pool is a pool scoped to spanned expressions.
pub type ExprPool = Pool<Inner>;

/// A wrapped expression where each child node is a reference to elsewhere in the pool.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(PoolRef<Inner>);

impl Expr {
    /// Inserts a new expression into the pool.
    pub fn insert(pool: &mut ExprPool, span: Option<Span>, expression: Expression<Expr>) -> Self {
        Self(pool.add(Inner { span, expression }))
    }

    /// Reads the entry from the pool.
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
