//! Evaluates a core AST.
//!
//! This evaluator first pools expressions into a vector, simplifying access.

pub mod ast;
mod evaluator;
mod pool;
mod pooler;

use boo_core::evaluation::EvaluationContext;

pub use evaluator::{NewInnerEvaluator, PoolingEvaluationContext, PoolingEvaluator};

pub fn new<NewInner: for<'pool> evaluator::NewInnerEvaluator<'pool>>() -> impl EvaluationContext {
    evaluator::PoolingEvaluationContext::<NewInner>::new()
}
