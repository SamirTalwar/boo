pub mod builders;
pub mod simple;

use std::fmt::Display;

use crate::identifier::Identifier;
use crate::operation::Operation;
use crate::primitive::Primitive;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression<Outer> {
    Primitive(Primitive),
    Identifier(Identifier),
    Assign(Assign<Outer>),
    Function(Function<Outer>),
    Apply(Apply<Outer>),
    Infix(Infix<Outer>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Assign<Outer> {
    pub name: Identifier,
    pub value: Outer,
    pub inner: Outer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function<Outer> {
    pub parameter: Identifier,
    pub body: Outer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Apply<Outer> {
    pub function: Outer,
    pub argument: Outer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Infix<Outer> {
    pub operation: Operation,
    pub left: Outer,
    pub right: Outer,
}

pub trait ExpressionWrapper
where
    Self: Sized,
{
    type Annotation;

    fn new(anotation: Self::Annotation, expression: Expression<Self>) -> Self;

    fn annotation(&self) -> Self::Annotation;

    fn expression(self) -> Expression<Self>;

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
    pub fn map<Next>(
        self,
        f: &mut impl FnMut(Outer::Annotation, Expression<Next>) -> Next,
    ) -> Expression<Next> {
        match self {
            Expression::Primitive(x) => Expression::Primitive(x),
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
