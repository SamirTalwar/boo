#![cfg(test)]

use super::ast::*;
use super::*;

use crate::identifier::*;
use crate::operation::*;
use crate::primitive::*;

pub fn primitive(pool: &mut ExprPool, value: Primitive) -> ExprRef {
    pool.add(Expression::Primitive { value })
}

pub fn primitive_integer(pool: &mut ExprPool, value: Integer) -> ExprRef {
    primitive(pool, Primitive::Integer(value))
}

pub fn identifier(pool: &mut ExprPool, name: Identifier) -> ExprRef {
    pool.add(Expression::Identifier { name })
}

pub fn identifier_string(pool: &mut ExprPool, name: String) -> ExprRef {
    identifier(pool, Identifier::new(name).unwrap())
}

pub fn assign(pool: &mut ExprPool, name: Identifier, value: ExprRef, inner: ExprRef) -> ExprRef {
    pool.add(Expression::Let { name, value, inner })
}

pub fn assign_string(pool: &mut ExprPool, name: String, value: ExprRef, inner: ExprRef) -> ExprRef {
    assign(pool, Identifier::new(name).unwrap(), value, inner)
}

pub fn infix(pool: &mut ExprPool, operation: Operation, left: ExprRef, right: ExprRef) -> ExprRef {
    pool.add(Expression::Infix {
        operation,
        left,
        right,
    })
}
