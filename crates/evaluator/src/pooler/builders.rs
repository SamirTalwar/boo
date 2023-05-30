#![cfg(test)]

use super::ast::*;

use boo_core::identifier::*;
use boo_core::operation::*;
use boo_core::primitive::*;
use boo_core::span::Spanned;

pub fn primitive(pool: &mut ExprPool, value: Primitive) -> ExprRef {
    pool.add(spanned(Expression::Primitive(value)))
}

pub fn primitive_integer(pool: &mut ExprPool, value: Integer) -> ExprRef {
    primitive(pool, Primitive::Integer(value))
}

pub fn identifier(pool: &mut ExprPool, name: Identifier) -> ExprRef {
    pool.add(spanned(Expression::Identifier(name)))
}

pub fn identifier_string(pool: &mut ExprPool, name: String) -> ExprRef {
    identifier(pool, Identifier::new(name).unwrap())
}

pub fn assign(pool: &mut ExprPool, name: Identifier, value: ExprRef, inner: ExprRef) -> ExprRef {
    pool.add(spanned(Expression::Assign(Assign { name, value, inner })))
}

pub fn assign_string(pool: &mut ExprPool, name: String, value: ExprRef, inner: ExprRef) -> ExprRef {
    assign(pool, Identifier::new(name).unwrap(), value, inner)
}

pub fn function(pool: &mut ExprPool, parameter: Identifier, body: ExprRef) -> ExprRef {
    pool.add(spanned(Expression::Function(Function { parameter, body })))
}

pub fn apply(pool: &mut ExprPool, function: ExprRef, argument: ExprRef) -> ExprRef {
    pool.add(spanned(Expression::Apply(Apply { function, argument })))
}

pub fn infix(pool: &mut ExprPool, operation: Operation, left: ExprRef, right: ExprRef) -> ExprRef {
    pool.add(spanned(Expression::Infix(Infix {
        operation,
        left,
        right,
    })))
}

fn spanned(value: Expression) -> Spanned<Expression> {
    Spanned {
        span: 0.into(),
        value,
    }
}
