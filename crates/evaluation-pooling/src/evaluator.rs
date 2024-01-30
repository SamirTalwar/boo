//! Pools [`Expr`][super::pooler::ast::Expr] values and evaluates them.

use std::marker::PhantomData;

use boo_core::error::*;
use boo_core::evaluation::*;
use boo_core::expr::Expr;
use boo_core::identifier::*;
use boo_evaluation_lazy::Bindings;

use crate::ast;
use crate::pooler::add_expr;

/// An expression pool together with its bound context.
pub struct PoolingEvaluationContext<NewInner: for<'pool> NewInnerEvaluator<'pool>> {
    pool_builder: ast::ExprPoolBuilder,
    bindings: Bindings<ast::Expr>,
    new_inner_marker: PhantomData<NewInner>,
}

impl<NewInner: for<'pool> NewInnerEvaluator<'pool>> PoolingEvaluationContext<NewInner> {
    pub fn new() -> Self {
        Self {
            pool_builder: ast::ExprPoolBuilder::new(),
            bindings: Bindings::new(),
            new_inner_marker: PhantomData,
        }
    }
}

impl<NewInner: for<'pool> NewInnerEvaluator<'pool>> Default for PoolingEvaluationContext<NewInner> {
    fn default() -> Self {
        Self::new()
    }
}

impl<NewInner: for<'pool> NewInnerEvaluator<'pool>> EvaluationContext
    for PoolingEvaluationContext<NewInner>
{
    type Eval = PoolingEvaluator<NewInner>;

    fn bind(&mut self, identifier: Identifier, expr: Expr) -> Result<()> {
        let pool_ref = add_expr(&mut self.pool_builder, expr);
        self.bindings = self.bindings.with(identifier, pool_ref, Bindings::new());
        Ok(())
    }

    fn evaluator(self) -> Self::Eval {
        PoolingEvaluator {
            pool: self.pool_builder.build(),
            bindings: self.bindings,
            new_inner_marker: PhantomData,
        }
    }
}

/// An expression pool together with its bound context.
/// We can use these to evaluate a given expression reference from the pool.
pub struct PoolingEvaluator<NewInner: for<'pool> NewInnerEvaluator<'pool>> {
    pool: ast::ExprPool,
    bindings: Bindings<ast::Expr>,
    new_inner_marker: PhantomData<NewInner>,
}

impl<NewInner: for<'pool> NewInnerEvaluator<'pool>> Evaluator for PoolingEvaluator<NewInner> {
    fn evaluate(&self, expr: Expr) -> Result<Evaluated> {
        let mut builder = self.pool.fork();
        let root = add_expr(&mut builder, expr);
        let fork = builder.build();
        let inner = NewInner::new(&fork, self.bindings.clone());
        inner.evaluate(root).map(|result| result.to_core(&fork))
    }
}

pub trait NewInnerEvaluator<'pool> {
    type Inner: Evaluator<ast::Expr>;

    fn new(pool: &'pool ast::ExprPool, bindings: Bindings<ast::Expr>) -> Self::Inner;
}
