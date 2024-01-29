//! Evaluates a core AST.
//!
//! This evaluator makes use of two optimizations:
//!
//!   1.  The AST is first "pooled" into a vector, simplifying access.
//!   2.  All bound variables are "thunked"; they will only be evaluated the
//!       first time they are read.

mod ast;
mod evaluator;
mod pool;
mod pooler;

pub use evaluator::PoolingEvaluator;
