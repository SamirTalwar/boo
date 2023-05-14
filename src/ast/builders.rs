#![cfg(test)]

use super::*;
use crate::primitive::*;

pub fn primitive<Annotation>(
    annotation: impl Into<Annotation>,
    value: Primitive,
) -> Expr<Annotation> {
    Expr::Primitive {
        annotation: annotation.into(),
        value,
    }
}

pub fn primitive_integer<Annotation>(
    annotation: impl Into<Annotation>,
    value: Integer,
) -> Expr<Annotation> {
    primitive(annotation, Primitive::Integer(value))
}

pub fn identifier<Annotation>(
    annotation: impl Into<Annotation>,
    name: Identifier,
) -> Expr<Annotation> {
    Expr::Identifier {
        annotation: annotation.into(),
        name,
    }
}

pub fn identifier_string<Annotation>(
    annotation: impl Into<Annotation>,
    name: String,
) -> Expr<Annotation> {
    identifier(annotation, Identifier::new(name).unwrap())
}

pub fn assign<Annotation>(
    annotation: impl Into<Annotation>,
    name: Identifier,
    value: Expr<Annotation>,
    inner: Expr<Annotation>,
) -> Expr<Annotation> {
    Expr::Let {
        annotation: annotation.into(),
        name,
        value: value.into(),
        inner: inner.into(),
    }
}

pub fn assign_string<Annotation>(
    annotation: impl Into<Annotation>,
    name: String,
    value: Expr<Annotation>,
    inner: Expr<Annotation>,
) -> Expr<Annotation> {
    assign(annotation, Identifier::new(name).unwrap(), value, inner)
}

pub fn infix<Annotation>(
    annotation: impl Into<Annotation>,
    operation: Operation,
    left: Expr<Annotation>,
    right: Expr<Annotation>,
) -> Expr<Annotation> {
    Expr::Infix {
        annotation: annotation.into(),
        operation,
        left: left.into(),
        right: right.into(),
    }
}
