//! Builders for a valid [`Expr`][super::Expr].

use boo_core::primitive::*;

use super::*;

pub fn primitive(span: impl Into<Span>, value: Primitive) -> Expr {
    Expr::new(span.into(), Expression::Primitive(value))
}

pub fn primitive_integer(span: impl Into<Span>, value: Integer) -> Expr {
    primitive(span, Primitive::Integer(value))
}

pub fn identifier(span: impl Into<Span>, name: Identifier) -> Expr {
    Expr::new(span.into(), Expression::Identifier(name))
}

pub fn function(span: impl Into<Span>, parameters: Vec<Identifier>, body: Expr) -> Expr {
    assert!(!parameters.is_empty(), "parameters must not be empty");
    Expr::new(
        span.into(),
        Expression::Function(Function { parameters, body }),
    )
}

pub fn apply(span: impl Into<Span>, function: Expr, argument: Expr) -> Expr {
    Expr::new(span.into(), Expression::Apply(Apply { function, argument }))
}

pub fn assign(span: impl Into<Span>, name: Identifier, value: Expr, inner: Expr) -> Expr {
    Expr::new(
        span.into(),
        Expression::Assign(Assign { name, value, inner }),
    )
}

pub fn infix(span: impl Into<Span>, operation: Operation, left: Expr, right: Expr) -> Expr {
    Expr::new(
        span.into(),
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }),
    )
}
