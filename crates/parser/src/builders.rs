use super::*;

use boo_core::ast::*;
use boo_core::identifier::Identifier;
use boo_core::operation::*;
use boo_core::primitive::*;
use boo_core::span::*;

pub fn primitive(span: impl Into<Span>, value: Primitive) -> Expr {
    wrapped(span, Expression::Primitive(value))
}

pub fn primitive_integer(span: impl Into<Span>, value: Integer) -> Expr {
    primitive(span, Primitive::Integer(value))
}

pub fn identifier(span: impl Into<Span>, name: Identifier) -> Expr {
    wrapped(span, Expression::Identifier(name))
}

pub fn identifier_string(span: impl Into<Span>, name: String) -> Expr {
    identifier(span, Identifier::new(name).unwrap())
}

pub fn assign(span: impl Into<Span>, name: Identifier, value: Expr, inner: Expr) -> Expr {
    wrapped(span, Expression::Assign(Assign { name, value, inner }))
}

pub fn assign_string(span: impl Into<Span>, name: String, value: Expr, inner: Expr) -> Expr {
    assign(span, Identifier::new(name).unwrap(), value, inner)
}

pub fn function(span: impl Into<Span>, parameter: Identifier, body: Expr) -> Expr {
    wrapped(span, Expression::Function(Function { parameter, body }))
}

pub fn apply(span: impl Into<Span>, function: Expr, argument: Expr) -> Expr {
    wrapped(span, Expression::Apply(Apply { function, argument }))
}

pub fn infix(span: impl Into<Span>, operation: Operation, left: Expr, right: Expr) -> Expr {
    wrapped(
        span,
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }),
    )
}

fn wrapped(span: impl Into<Span>, value: Expression<Expr>) -> Expr {
    Expr(
        Spanned {
            span: span.into(),
            value,
        }
        .into(),
    )
}
