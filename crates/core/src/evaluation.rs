use crate::ast::Function;
use crate::error::Result;
use crate::expr::Expr;
use crate::primitive::Primitive;

pub trait Evaluator {
    fn evaluate(&self, expr: Expr) -> Result<Evaluated>;
}

/// An evaluation result. This can be either a primitive value or a closure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evaluated {
    Primitive(Primitive),
    Function(Function<Expr>),
}

impl std::fmt::Display for Evaluated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluated::Primitive(x) => x.fmt(f),
            Evaluated::Function(x) => x.fmt(f),
        }
    }
}
