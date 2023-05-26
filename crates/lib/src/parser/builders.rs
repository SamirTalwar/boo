#![cfg(test)]

use super::*;

use crate::identifier::Identifier;
use crate::primitive::*;

pub fn primitive(annotation: impl Into<Span>, value: Primitive) -> Expr {
    Spanned {
        span: annotation.into(),
        value: Expression::Primitive(value),
    }
    .into()
}

pub fn primitive_integer(annotation: impl Into<Span>, value: Integer) -> Expr {
    primitive(annotation, Primitive::Integer(value))
}

pub fn identifier(annotation: impl Into<Span>, name: Identifier) -> Expr {
    Spanned {
        span: annotation.into(),
        value: Expression::Identifier(name),
    }
    .into()
}

pub fn identifier_string(annotation: impl Into<Span>, name: String) -> Expr {
    identifier(annotation, Identifier::new(name).unwrap())
}

pub fn assign(annotation: impl Into<Span>, name: Identifier, value: Expr, inner: Expr) -> Expr {
    Spanned {
        span: annotation.into(),
        value: Expression::Assign(Assign { name, value, inner }),
    }
    .into()
}

pub fn assign_string(annotation: impl Into<Span>, name: String, value: Expr, inner: Expr) -> Expr {
    assign(annotation, Identifier::new(name).unwrap(), value, inner)
}

pub fn function(annotation: impl Into<Span>, parameter: Identifier, body: Expr) -> Expr {
    Spanned {
        span: annotation.into(),
        value: Expression::Function(Function { parameter, body }),
    }
    .into()
}

pub fn apply(annotation: impl Into<Span>, function: Expr, argument: Expr) -> Expr {
    Spanned {
        span: annotation.into(),
        value: Expression::Apply(Apply { function, argument }),
    }
    .into()
}

pub fn infix(annotation: impl Into<Span>, operation: Operation, left: Expr, right: Expr) -> Expr {
    Spanned {
        span: annotation.into(),
        value: Expression::Infix(Infix {
            operation,
            left,
            right,
        }),
    }
    .into()
}
