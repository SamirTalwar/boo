//! The AST produced by the parser.

pub mod builders;
pub mod operation;
mod rewriter;

use boo_core::error::Result;
use boo_core::identifier::Identifier;
use boo_core::primitive::Primitive;
use boo_core::span::Span;
use boo_core::verification;

pub use crate::operation::Operation;

/// An outer Boo language expression node, annotated with the source location.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    pub span: Span,
    pub expression: Box<Expression>,
}

impl Expr {
    /// Creates a new Boo language outer expression, given the inner expression.
    pub fn new(span: Span, expression: Expression) -> Self {
        Self {
            span,
            expression: expression.into(),
        }
    }

    /// Convert the expression to a core expression.
    pub fn to_core(self) -> Result<boo_core::expr::Expr> {
        let result = rewriter::rewrite(self)?;
        verification::verify(&result)?;
        Ok(result)
    }
}

/// An inner Boo language expression node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Primitive(Primitive),
    Identifier(Identifier),
    Function(Function),
    Apply(Apply),
    Assign(Assign),
    Match(Match),
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
    /// The names of the function parameters.
    pub parameters: Vec<Identifier>,
    /// The body of the function.
    pub body: Expr,
}

/// A set of patterns matched against a value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Match {
    /// The value to be matched.
    pub value: Expr,
    /// The patterns.
    pub patterns: Vec<PatternMatch>,
}

/// A single pattern and its assigned result.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatternMatch {
    /// The pattern to be matched.
    pub pattern: Pattern,
    /// The result of matching against the pattern.
    pub result: Expr,
}

/// A single pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    Anything,
    Primitive(Primitive),
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
            Expression::Function(x) => x.fmt(f),
            Expression::Apply(x) => x.fmt(f),
            Expression::Assign(x) => x.fmt(f),
            Expression::Match(x) => x.fmt(f),
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
        write!(f, "fn ")?;
        for parameter in &self.parameters {
            write!(f, "{} ", parameter)?;
        }
        write!(f, "-> ({})", self.body)
    }
}

impl std::fmt::Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "match {} {{", self.value)?;
        let mut pattern_iter = self.patterns.iter();
        if let Some(PatternMatch {
            pattern: first_pattern,
            result: first_result,
        }) = pattern_iter.next()
        {
            write!(f, "{} -> ({})", first_pattern, first_result)?;
            for PatternMatch { pattern, result } in pattern_iter {
                write!(f, "; {} -> ({})", pattern, result)?;
            }
        }
        write!(f, "}}")
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Anything => write!(f, "_"),
            Pattern::Primitive(x) => x.fmt(f),
        }
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
