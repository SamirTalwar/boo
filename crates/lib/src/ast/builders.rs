#![cfg(test)]

use super::*;
use crate::primitive::*;

pub fn primitive<Annotation>(
    annotation: impl Into<Annotation>,
    value: Primitive,
) -> Expr<Annotation> {
    Annotated {
        annotation: annotation.into(),
        value: Expression::Primitive { value },
    }
    .into()
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
    Annotated {
        annotation: annotation.into(),
        value: Expression::Identifier { name },
    }
    .into()
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
    Annotated {
        annotation: annotation.into(),
        value: Expression::Let { name, value, inner },
    }
    .into()
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
    Annotated {
        annotation: annotation.into(),
        value: Expression::Infix {
            operation,
            left,
            right,
        },
    }
    .into()
}
