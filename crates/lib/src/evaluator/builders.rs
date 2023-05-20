#![cfg(test)]

use super::ast::*;
use super::*;

use crate::identifier::Identifier;
use crate::primitive::*;

pub fn primitive(value: Primitive) -> Expr {
    Expression::Primitive { value }.into()
}

pub fn primitive_integer(value: Integer) -> Expr {
    primitive(Primitive::Integer(value))
}

pub fn identifier(name: Identifier) -> Expr {
    Expression::Identifier { name }.into()
}

pub fn identifier_string(name: String) -> Expr {
    identifier(Identifier::new(name).unwrap())
}

pub fn assign(name: Identifier, value: Expr, inner: Expr) -> Expr {
    Expression::Let { name, value, inner }.into()
}

pub fn assign_string(name: String, value: Expr, inner: Expr) -> Expr {
    assign(Identifier::new(name).unwrap(), value, inner)
}

pub fn infix(operation: Operation, left: Expr, right: Expr) -> Expr {
    Expression::Infix {
        operation,
        left,
        right,
    }
    .into()
}
