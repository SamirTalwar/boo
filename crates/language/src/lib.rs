//! The AST produced by the parser.

pub mod builders;
pub mod operation;
mod rewriter;

use boo_core::identifier::Identifier;
use boo_core::primitive::Primitive;
use boo_core::span::Span;

pub use crate::operation::Operation;

/// An expression wrapper, annotated with the source location as a span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    pub span: Span,
    pub expression: Box<Expression>,
}

impl Expr {
    pub fn new(span: Span, expression: Expression) -> Self {
        Self {
            span,
            expression: expression.into(),
        }
    }

    /// Convert the expression to a core expression.
    pub fn to_core(self) -> boo_core::expr::Expr {
        rewriter::rewrite(self)
    }
}

/// A Boo expression. These can be nested arbitrarily.
///
/// This cannot be used on its own; it must be used with [`Expr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Primitive(Primitive),
    Identifier(Identifier),
    Assign(Assign),
    Function(Function),
    Apply(Apply),
    Infix(Infix),
}

/// Represents assignment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Assign {
    /// The name of the assigned variable.
    pub name: Identifier,
    /// The value of the assigned variable.
    pub value: Expr,
    /// The rest of the expression.
    pub inner: Expr,
}

/// Represents a function definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    /// The name of the function parameter.
    pub parameter: Identifier,
    /// The body of the function.
    pub body: Expr,
}

/// Applies an argument to a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Apply {
    /// The function.
    pub function: Expr,
    /// The argument.
    pub argument: Expr,
}

/// An infix operation on integers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Infix {
    /// The operation.
    pub operation: Operation,
    /// The left operand.
    pub left: Expr,
    /// The right operand.
    pub right: Expr,
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.expression.fmt(f)
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Primitive(x) => x.fmt(f),
            Expression::Identifier(x) => x.fmt(f),
            Expression::Assign(x) => x.fmt(f),
            Expression::Function(x) => x.fmt(f),
            Expression::Apply(x) => x.fmt(f),
            Expression::Infix(x) => x.fmt(f),
        }
    }
}

impl std::fmt::Display for Assign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "let {} = ({}) in ({})",
            self.name, self.value, self.inner
        )
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {} -> ({})", self.parameter, self.body)
    }
}

impl std::fmt::Display for Apply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) ({})", self.function, self.argument)
    }
}

impl std::fmt::Display for Infix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {} ({})", self.left, self.operation, self.right)
    }
}
