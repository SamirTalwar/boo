//! Builders that insert nodes into a [`Pool`].

#![cfg(test)]

use super::ast::*;

use boo_core::ast::*;
use boo_core::identifier::*;
use boo_core::operation::*;
use boo_core::primitive::*;

pub fn primitive(pool: &mut ExprPool, value: Primitive) -> Expr {
    Expr::insert(pool, 0.into(), Expression::Primitive(value))
}

pub fn primitive_integer(pool: &mut ExprPool, value: Integer) -> Expr {
    primitive(pool, Primitive::Integer(value))
}

pub fn identifier(pool: &mut ExprPool, name: Identifier) -> Expr {
    Expr::insert(pool, 0.into(), Expression::Identifier(name))
}

pub fn identifier_string(pool: &mut ExprPool, name: String) -> Expr {
    identifier(pool, Identifier::name_from_string(name).unwrap())
}

pub fn assign(pool: &mut ExprPool, name: Identifier, value: Expr, inner: Expr) -> Expr {
    Expr::insert(
        pool,
        0.into(),
        Expression::Assign(Assign { name, value, inner }),
    )
}

pub fn assign_string(pool: &mut ExprPool, name: String, value: Expr, inner: Expr) -> Expr {
    assign(
        pool,
        Identifier::name_from_string(name).unwrap(),
        value,
        inner,
    )
}

pub fn function(pool: &mut ExprPool, parameter: Identifier, body: Expr) -> Expr {
    Expr::insert(
        pool,
        0.into(),
        Expression::Function(Function { parameter, body }),
    )
}

pub fn apply(pool: &mut ExprPool, function: Expr, argument: Expr) -> Expr {
    Expr::insert(
        pool,
        0.into(),
        Expression::Apply(Apply { function, argument }),
    )
}

pub fn infix(pool: &mut ExprPool, operation: Operation, left: Expr, right: Expr) -> Expr {
    Expr::insert(
        pool,
        0.into(),
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }),
    )
}
