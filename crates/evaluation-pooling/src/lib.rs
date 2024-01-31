//! Evaluates a core AST.
//!
//! This evaluator first pools expressions into a vector, simplifying access.

pub mod ast;
mod evaluator;
mod pool;
mod pooler;

use boo_core::evaluation::Evaluator;
pub use evaluator::{NewInnerEvaluator, PoolingEvaluator};

pub fn new<NewInner: for<'pool> NewInnerEvaluator<'pool>>() -> impl Evaluator {
    PoolingEvaluator::<NewInner>::new()
}
