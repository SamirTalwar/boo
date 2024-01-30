//! Pools [`Expr`][super::pooler::ast::Expr] values and evaluates them.

use std::marker::PhantomData;

use boo_core::error::*;
use boo_core::evaluation::*;
use boo_core::expr::Expr;
use boo_core::identifier::*;
use boo_evaluation_lazy::Bindings;

use crate::ast;
use crate::pooler::add_expr;

/// An expression pool together with the current bound context, which can
/// evaluate a given expression reference from the pool.
pub struct PoolingEvaluator<NewInner: for<'pool> NewInnerEvaluator<'pool>> {
    pool: ast::ExprPool,
    bindings: Bindings<ast::Expr>,
    new_inner_marker: PhantomData<NewInner>,
}

impl<NewInner: for<'pool> NewInnerEvaluator<'pool>> PoolingEvaluator<NewInner> {
    pub fn new() -> Self {
        Self {
            pool: ast::ExprPool::new(),
            bindings: Bindings::new(),
            new_inner_marker: PhantomData,
        }
    }
}

impl<NewInner: for<'pool> NewInnerEvaluator<'pool>> Default for PoolingEvaluator<NewInner> {
    fn default() -> Self {
        Self::new()
    }
}

impl<NewInner: for<'pool> NewInnerEvaluator<'pool>> EvaluationContext
    for PoolingEvaluator<NewInner>
{
    type Eval = Self;

    fn bind(&mut self, identifier: Identifier, expr: Expr) -> Result<()> {
        let pool_ref = add_expr(&mut self.pool, expr);
        self.bindings = self.bindings.with(identifier, pool_ref, Bindings::new());
        Ok(())
    }

    fn evaluator(self) -> Self::Eval {
        self
    }
}

impl<NewInner: for<'pool> NewInnerEvaluator<'pool>> Evaluator for PoolingEvaluator<NewInner> {
    fn evaluate(&self, expr: Expr) -> Result<Evaluated> {
        let mut pool = self.pool.clone();
        let root = add_expr(&mut pool, expr);
        let inner = NewInner::new(&pool, self.bindings.clone());
        inner.evaluate(root).map(|result| result.to_core(&pool))
    }
}

pub trait NewInnerEvaluator<'pool> {
    type Inner: Evaluator<ast::Expr>;

    fn new(pool: &'pool ast::ExprPool, bindings: Bindings<ast::Expr>) -> Self::Inner;
}
