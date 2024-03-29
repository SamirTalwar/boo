use boo_core::evaluation::EvaluationContext;
use boo_evaluation_lazy::Bindings;
use boo_evaluation_pooling::{ast, NewInnerEvaluator, PoolingEvaluationContext};

pub fn new() -> impl EvaluationContext {
    PoolingEvaluationContext::<NewRecursiveEvaluator>::new()
}

pub struct NewRecursiveEvaluator {}

impl<'pool> NewInnerEvaluator<'pool> for NewRecursiveEvaluator {
    type Inner = boo_evaluation_recursive::RecursiveEvaluator<ast::Expr, &'pool ast::ExprPool>;

    fn new(pool: &'pool ast::ExprPool, bindings: Bindings<ast::Expr>) -> Self::Inner {
        boo_evaluation_recursive::RecursiveEvaluator::new(pool, bindings)
    }
}
