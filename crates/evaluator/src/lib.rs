//! Evaluates a parsed AST.
//!
//! This evaluator makes use of two optimizations:
//!
//!   1.  The AST is first "pooled" into a vector, simplifying access.
//!   2.  All bound variables are "thunked"; they will only be evaluated the
//!       first time they are read.

pub mod evaluator;
pub mod pooler;
pub mod thunk;

pub use evaluator::Evaluated;

use boo_core::ast::ExpressionWrapper;
use boo_core::error::Result;
use boo_core::span::Span;

/// Evaluates a parsed expression.
pub fn evaluate<Expr>(expr: Expr) -> Result<evaluator::Evaluated>
where
    Expr: ExpressionWrapper<Annotation = Span> + Clone,
{
    let (pool, root) = pooler::pool_exprs(expr);
    evaluator::evaluate(&pool, root)
}
