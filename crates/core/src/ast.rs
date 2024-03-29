//! Structures that make up the core Boo AST.

use std::collections::VecDeque;
use std::fmt::Display;

use crate::identifier::Identifier;
use crate::native::Native;
use crate::primitive::Primitive;
use crate::types::Monotype;

/// A Boo expression. These can be nested arbitrarily.
///
/// This cannot be used on its own; it must be used with a wrapper `struct`. The
/// simplest wraps the expression in a `Box`:
///
/// ```
/// # use boo_core::ast::Expression;
/// struct Expr(Box<Expression<Expr>>);
/// ```
///
/// This allows us to share some common data structures across the stages of the
/// interpreter.
///
/// Note that this must be a `struct` and not a type alias to allow for
/// type-level recursion.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression<Outer> {
    Primitive(Primitive),
    Native(Native),
    Identifier(Identifier),
    Function(Function<Outer>),
    Apply(Apply<Outer>),
    Assign(Assign<Outer>),
    Match(Match<Outer>),
    Typed(Typed<Outer>),
}

impl<Outer> AsRef<Expression<Outer>> for Expression<Outer> {
    fn as_ref(&self) -> &Expression<Outer> {
        self
    }
}

/// Represents a function definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function<Outer> {
    /// The name of the function parameter.
    pub parameter: Identifier,
    /// The body of the function.
    pub body: Outer,
}

/// Applies an argument to a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Apply<Outer> {
    /// The function.
    pub function: Outer,
    /// The argument.
    pub argument: Outer,
}

/// Represents assignment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Assign<Outer> {
    /// The name of the assigned variable.
    pub name: Identifier,
    /// The value of the assigned variable.
    pub value: Outer,
    /// The rest of the expression.
    pub inner: Outer,
}

/// A set of patterns matched against a value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Match<Outer> {
    /// The value to be matched.
    pub value: Outer,
    /// The patterns.
    pub patterns: VecDeque<PatternMatch<Outer>>,
}

/// A single pattern and its assigned result.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PatternMatch<Outer> {
    /// The pattern to be matched.
    pub pattern: Pattern,
    /// The result of matching against the pattern.
    pub result: Outer,
}

/// An expression annotated with a type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Typed<Outer> {
    /// The expression.
    pub expression: Outer,
    /// The stated type of the expression.
    pub typ: Monotype,
}

/// A single pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    Anything,
    Primitive(Primitive),
}

impl<Outer: Display> Display for Expression<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Primitive(x) => x.fmt(f),
            Expression::Native(x) => x.fmt(f),
            Expression::Identifier(x) => x.fmt(f),
            Expression::Function(x) => x.fmt(f),
            Expression::Apply(x) => x.fmt(f),
            Expression::Assign(x) => x.fmt(f),
            Expression::Match(x) => x.fmt(f),
            Expression::Typed(x) => x.fmt(f),
        }
    }
}

impl<Outer: Display> Display for Function<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {} -> ({})", self.parameter, self.body)
    }
}

impl<Outer: Display> Display for Apply<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) ({})", self.function, self.argument)
    }
}

impl<Outer: Display> Display for Assign<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "let {} = ({}) in ({})",
            self.name, self.value, self.inner
        )
    }
}

impl<Outer: Display> Display for Match<Outer> {
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

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pattern::Primitive(x) => x.fmt(f),
            Pattern::Anything => write!(f, "_"),
        }
    }
}

impl<Outer: Display> Display for Typed<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}): {}", self.expression, self.typ)
    }
}
