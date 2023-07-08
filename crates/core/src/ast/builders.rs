//! Builders for any valid [`ExpressionWrapper`].

use crate::primitive::*;

use super::*;

pub fn primitive<Expr: ExpressionWrapper>(
    annotation: impl Into<Expr::Annotation>,
    value: Primitive,
) -> Expr {
    Expr::new(annotation.into(), Expression::Primitive(value))
}

pub fn primitive_integer<Expr: ExpressionWrapper>(
    annotation: impl Into<Expr::Annotation>,
    value: Integer,
) -> Expr {
    primitive(annotation, Primitive::Integer(value))
}

pub fn identifier<Expr: ExpressionWrapper>(
    annotation: impl Into<Expr::Annotation>,
    name: Identifier,
) -> Expr {
    Expr::new(annotation.into(), Expression::Identifier(name))
}

pub fn identifier_string<Expr: ExpressionWrapper>(
    annotation: impl Into<Expr::Annotation>,
    name: String,
) -> Expr {
    identifier(annotation, Identifier::name_from_string(name).unwrap())
}

pub fn assign<Expr: ExpressionWrapper>(
    annotation: impl Into<Expr::Annotation>,
    name: Identifier,
    value: Expr,
    inner: Expr,
) -> Expr {
    Expr::new(
        annotation.into(),
        Expression::Assign(Assign { name, value, inner }),
    )
}

pub fn assign_string<Expr: ExpressionWrapper>(
    annotation: impl Into<Expr::Annotation>,
    name: String,
    value: Expr,
    inner: Expr,
) -> Expr {
    assign(
        annotation,
        Identifier::name_from_string(name).unwrap(),
        value,
        inner,
    )
}

pub fn function<Expr: ExpressionWrapper>(
    annotation: impl Into<Expr::Annotation>,
    parameter: Identifier,
    body: Expr,
) -> Expr {
    Expr::new(
        annotation.into(),
        Expression::Function(Function { parameter, body }),
    )
}

pub fn apply<Expr: ExpressionWrapper>(
    annotation: impl Into<Expr::Annotation>,
    function: Expr,
    argument: Expr,
) -> Expr {
    Expr::new(
        annotation.into(),
        Expression::Apply(Apply { function, argument }),
    )
}

pub fn infix<Expr: ExpressionWrapper>(
    annotation: impl Into<Expr::Annotation>,
    operation: Operation,
    left: Expr,
    right: Expr,
) -> Expr {
    Expr::new(
        annotation.into(),
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }),
    )
}
