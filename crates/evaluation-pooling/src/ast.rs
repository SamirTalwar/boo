//! An AST based on a [`Pool`].

use boo_core::ast::Expression;
use boo_core::evaluation::ExpressionReader;
use boo_core::span::{Span, Spanned};

use super::pool::*;

pub type Inner = Spanned<Expression<Expr>>;

/// A wrapped expression where each child node is a reference to elsewhere in the pool.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(PoolRef<Inner>);

impl Expr {
    /// Inserts a new expression into the pool.
    pub fn insert(
        builder: &mut ExprPoolBuilder,
        span: Option<Span>,
        expression: Expression<Expr>,
    ) -> Self {
        Self(builder.add(Spanned {
            span,
            value: expression,
        }))
    }

    /// Reads the entry from the pool.
    pub fn read_from<'a>(&self, pool: &'a ExprPool) -> &'a Inner {
        pool.get(self.0)
    }
}

impl Copy for Expr {}

/// A builder for [ExprPool][].
pub type ExprPoolBuilder = PoolBuilder<Inner>;

/// An expression pool is a pool scoped to spanned expressions.
pub type ExprPool = Pool<Inner>;

impl<'a> ExpressionReader for &'a ExprPool {
    type Expr = self::Expr;
    type Target = &'a Expression<Self::Expr>;

    fn read(&self, expr: Self::Expr) -> Spanned<Self::Target> {
        expr.read_from(self).as_ref()
    }
}
