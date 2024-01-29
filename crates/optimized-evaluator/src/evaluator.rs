//! Pools [`Expr`][super::pooler::ast::Expr] values and evaluates them.

use std::collections;

use boo_core::error::*;
use boo_core::evaluation::*;
use boo_core::expr::Expr as CoreExpr;
use boo_core::identifier::*;
use boo_evaluation_lazy::Bindings;
use boo_evaluation_recursive::RecursiveEvaluator;

use crate::ast;
use crate::pooler::add_expr;

/// An expression pool together with the current bound context, which can
/// evaluate a given expression reference from the pool.
pub struct PoolingEvaluator {
    pool: ast::ExprPool,
    bindings: collections::HashMap<Identifier, ast::Expr>,
}

impl PoolingEvaluator {
    pub fn new() -> Self {
        Self {
            pool: ast::ExprPool::new(),
            bindings: collections::HashMap::new(),
        }
    }
}

impl Default for PoolingEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for PoolingEvaluator {
    fn bind(&mut self, identifier: Identifier, expr: CoreExpr) -> Result<()> {
        let pool_ref = add_expr(&mut self.pool, expr);
        self.bindings.insert(identifier, pool_ref);
        Ok(())
    }

    fn evaluate(&self, expr: CoreExpr) -> Result<Evaluated> {
        let mut pool = self.pool.clone();
        let root = add_expr(&mut pool, expr);
        let bindings =
            self.bindings
                .iter()
                .fold(Bindings::new(), |bindings, (identifier, pool_ref)| {
                    bindings.with(identifier.clone(), *pool_ref, Bindings::new())
                });
        let inner = RecursiveEvaluator::new(&pool, bindings);
        inner.evaluate(root).map(|progress| progress.finish(&pool))
    }
}
