//! Represents the result of evaluating an expression.

use boo_core::evaluation::Evaluated;
use boo_core::expr::Function;
use boo_core::identifier::Identifier;
use boo_core::primitive::Primitive;

use crate::bindings::Bindings;

/// An interim evaluation result.
#[derive(Debug, Clone)]
pub enum CompletedEvaluation<Expr: Clone> {
    Primitive(Primitive),
    Closure {
        parameter: Identifier,
        body: Expr,
        bindings: Bindings<Expr>,
    },
}

impl<Expr: Clone> CompletedEvaluation<Expr> {
    /// Concludes evaluation.
    pub fn finish(self) -> Evaluated<Expr> {
        match self {
            Self::Primitive(primitive) => Evaluated::Primitive(primitive),
            Self::Closure {
                parameter,
                body,
                bindings: _,
            } => Evaluated::Function(Function { parameter, body }),
        }
    }
}
