//! The core Boo AST, represented as a wrapped [`Expression`].

pub mod builders;
pub mod simple;

use std::fmt::Display;

use crate::identifier::Identifier;
use crate::native::Native;
use crate::operation::Operation;
use crate::primitive::Primitive;

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
    Assign(Assign<Outer>),
    Function(Function<Outer>),
    Apply(Apply<Outer>),
    Infix(Infix<Outer>),
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

/// An infix operation on integers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Infix<Outer> {
    /// The operation.
    pub operation: Operation,
    /// The left operand.
    pub left: Outer,
    /// The right operand.
    pub right: Outer,
}

/// Denotes a wrapper for `Expression` that allows for transformation into
/// a different wrapper.
///
/// This is used to allow for operations that can work on the various stages of
/// AST, e.g. parsing, type-checking, pooling, and finally evaluation.
pub trait ExpressionWrapper
where
    Self: Sized,
{
    type Annotation;

    /// Constructs a new wrapper.
    fn new(annotation: Self::Annotation, expression: Expression<Self>) -> Self;

    /// Constructs a new wrapper with a default annotation value.
    ///
    /// This should not be used by the interpreter, but can be used by
    /// synthesis.
    fn new_unannotated(expression: Expression<Self>) -> Self;

    /// Acquires a copy of the annotation.
    fn annotation(&self) -> Self::Annotation;

    /// Unwraps the expression.
    fn expression(self) -> Expression<Self>;

    /// Transforms the wrapper, while keeping the expression tree intact.
    fn transform<Next>(
        self,
        f: &mut impl FnMut(Self::Annotation, Expression<Next>) -> Next,
    ) -> Next {
        let annotation = self.annotation();
        let mapped = self.expression().map(f);
        f(annotation, mapped)
    }
}

impl<Outer: ExpressionWrapper> Expression<Outer> {
    /// Used along with `ExpressionWrapper` to propagate a transformation
    /// through the tree.
    pub fn map<Next>(
        self,
        f: &mut impl FnMut(Outer::Annotation, Expression<Next>) -> Next,
    ) -> Expression<Next> {
        match self {
            Expression::Primitive(x) => Expression::Primitive(x),
            Expression::Native(x) => Expression::Native(x),
            Expression::Identifier(x) => Expression::Identifier(x),
            Expression::Assign(Assign { name, value, inner }) => Expression::Assign(Assign {
                name,
                value: value.transform(f),
                inner: inner.transform(f),
            }),
            Expression::Function(Function { parameter, body }) => Expression::Function(Function {
                parameter,
                body: body.transform(f),
            }),
            Expression::Apply(Apply { function, argument }) => Expression::Apply(Apply {
                function: function.transform(f),
                argument: argument.transform(f),
            }),
            Expression::Infix(Infix {
                operation,
                left,
                right,
            }) => Expression::Infix(Infix {
                operation,
                left: left.transform(f),
                right: right.transform(f),
            }),
        }
    }
}

impl<Outer: Display> std::fmt::Display for Expression<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Primitive(x) => x.fmt(f),
            Expression::Native(x) => x.fmt(f),
            Expression::Identifier(x) => x.fmt(f),
            Expression::Assign(x) => x.fmt(f),
            Expression::Function(x) => x.fmt(f),
            Expression::Apply(x) => x.fmt(f),
            Expression::Infix(x) => x.fmt(f),
        }
    }
}

impl<Outer: Display> std::fmt::Display for Assign<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "let {} = ({}) in ({})",
            self.name, self.value, self.inner
        )
    }
}

impl<Outer: Display> std::fmt::Display for Function<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {} -> ({})", self.parameter, self.body)
    }
}

impl<Outer: Display> std::fmt::Display for Apply<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) ({})", self.function, self.argument)
    }
}

impl<Outer: Display> std::fmt::Display for Infix<Outer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {} ({})", self.left, self.operation, self.right)
    }
}
