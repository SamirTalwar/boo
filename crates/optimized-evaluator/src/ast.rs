//! An AST based on a [`Pool`].

use boo_core::ast::Expression;
use boo_core::span::{Span, Spanned};

use super::pool::*;

pub type Inner = Spanned<Expression<Expr>>;

/// A wrapped expression where each child node is a reference to elsewhere in the pool.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(PoolRef<Inner>);

impl Expr {
    /// Inserts a new expression into the pool.
    pub fn insert(pool: &mut ExprPool, span: Option<Span>, expression: Expression<Expr>) -> Self {
        Self(pool.add(span, expression))
    }

    /// Reads the entry from the pool.
    pub fn read_from<'a>(&self, pool: &'a ExprPool) -> &'a Inner {
        pool.get(self.0)
    }
}

impl Copy for Expr {}

/// An expression pool is a pool scoped to spanned expressions.
#[derive(Clone)]
pub struct ExprPool(Pool<Inner>);

impl ExprPool {
    pub fn new() -> Self {
        Self(Pool::new())
    }

    fn add(&mut self, span: Option<Span>, expression: Expression<Expr>) -> PoolRef<Inner> {
        self.0.add(Spanned {
            span,
            value: expression,
        })
    }

    fn get(&self, pool_ref: PoolRef<Inner>) -> &Inner {
        self.0.get(pool_ref)
    }
}
