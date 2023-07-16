use crate::ast::Function;
use crate::error::Result;
use crate::expr::Expr;
use crate::identifier::Identifier;
use crate::primitive::Primitive;

/// An evaluator knows how to evaluate expressions within a context.
///
/// Context can be added in the form of top-level bindings to other expressions.
pub trait Evaluator {
    /// Bind a new top-level expression.
    fn bind(&mut self, identifier: Identifier, expr: Expr) -> Result<()>;

    /// Evaluate the given expression.
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
