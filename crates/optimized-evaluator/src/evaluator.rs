//! Pools [`Expr`][super::pooler::ast::Expr] values and evaluates them.

use std::collections;
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
    bindings: collections::HashMap<Identifier, ast::Expr>,
    new_inner_marker: PhantomData<NewInner>,
}

impl PoolingEvaluator<NewRecursiveEvaluator> {
    pub fn new_recursive() -> Self {
        Self::new()
    }
}

impl<NewInner: for<'pool> NewInnerEvaluator<'pool>> PoolingEvaluator<NewInner> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            pool: ast::ExprPool::new(),
            bindings: collections::HashMap::new(),
            new_inner_marker: PhantomData,
        }
    }
}

impl<NewInner: for<'pool> NewInnerEvaluator<'pool>> Evaluator for PoolingEvaluator<NewInner> {
    fn bind(&mut self, identifier: Identifier, expr: Expr) -> Result<()> {
        let pool_ref = add_expr(&mut self.pool, expr);
        self.bindings.insert(identifier, pool_ref);
        Ok(())
    }

    fn evaluate(&self, expr: Expr) -> Result<Evaluated> {
        let mut pool = self.pool.clone();
        let root = add_expr(&mut pool, expr);
        let bindings =
            self.bindings
                .iter()
                .fold(Bindings::new(), |bindings, (identifier, pool_ref)| {
                    bindings.with(identifier.clone(), *pool_ref, Bindings::new())
                });
        let inner = NewInner::new(&pool, bindings);
        inner.evaluate(root).map(|result| result.to_core(&pool))
    }
}

pub trait NewInnerEvaluator<'pool> {
    type Inner: Evaluator<ast::Expr>;

    fn new(pool: &'pool ast::ExprPool, bindings: Bindings<ast::Expr>) -> Self::Inner;
}

pub struct NewRecursiveEvaluator {}

impl<'pool> NewInnerEvaluator<'pool> for NewRecursiveEvaluator {
    type Inner = boo_evaluation_recursive::RecursiveEvaluator<ast::Expr, &'pool ast::ExprPool>;

    fn new(pool: &'pool ast::ExprPool, bindings: Bindings<ast::Expr>) -> Self::Inner {
        boo_evaluation_recursive::RecursiveEvaluator::new(pool, bindings)
    }
}
