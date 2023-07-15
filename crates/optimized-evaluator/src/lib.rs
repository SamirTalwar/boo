//! Evaluates a core AST.
//!
//! This evaluator makes use of two optimizations:
//!
//!   1.  The AST is first "pooled" into a vector, simplifying access.
//!   2.  All bound variables are "thunked"; they will only be evaluated the
//!       first time they are read.

pub mod ast;
pub mod evaluator;
pub mod pool;
pub mod pooler;
pub mod thunk;

use boo_core::error::Result;
use boo_core::evaluation::*;
use boo_core::expr::Expr;

/// Evaluates a core AST.
pub struct OptimizedEvaluator {}

impl OptimizedEvaluator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for OptimizedEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for OptimizedEvaluator {
    fn evaluate(&self, expr: Expr) -> Result<Evaluated> {
        let (pool, root) = pooler::pool_exprs(expr);
        evaluator::evaluate(&pool, root)
    }
}
