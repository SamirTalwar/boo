use std::fmt::Display;

use crate::identifier::Identifier;
use crate::operation::Operation;
use crate::primitive::Primitive;

#[macro_export]
macro_rules! expr {
    ($wrapper:tt) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct Expr(pub boo_fill_hole::fill_hole!($wrapper, ($crate::ast::Expression<Expr>)));

        impl std::fmt::Display for Expr {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

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
