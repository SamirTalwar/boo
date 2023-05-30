pub mod evaluator;
pub mod pooler;
pub mod thunk;

pub use evaluator::Evaluated;

use boo_core::error::Result;

pub fn evaluate(expr: boo_parser::Expr) -> Result<evaluator::Evaluated> {
    let pool = pooler::pool_exprs(expr);
    evaluator::evaluate(&pool)
}
