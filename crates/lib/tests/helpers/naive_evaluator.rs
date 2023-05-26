use std::fmt::Display;

use im::HashMap;

use boo::error::*;
use boo::identifier::*;
use boo::operation::*;
use boo::parser::ast::*;
use boo::primitive::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evaluated {
    Primitive(Primitive),
}

impl Display for Evaluated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(primitive) => primitive.fmt(f),
        }
    }
}

pub fn naively_evaluate(expr: Expr) -> Result<Evaluated> {
    evaluate_(expr, HashMap::new())
}

#[allow(clippy::boxed_local)]
fn evaluate_(expr: Expr, assignments: HashMap<Identifier, Expr>) -> Result<Evaluated> {
    match &expr.value {
        Expression::Primitive(value) => Ok(Evaluated::Primitive(value.clone())),
        Expression::Identifier(name) => match assignments.get(name) {
            Some(value) => evaluate_(value.clone(), assignments),
            None => Err(Error::UnknownVariable {
                span: expr.span,
                name: name.to_string(),
            }),
        },
        Expression::Assign(Assign { name, value, inner }) => evaluate_(
            inner.clone(),
            assignments.update(name.clone(), value.clone()),
        ),
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }) => {
            let left_result = evaluate_(left.clone(), assignments.clone())?;
            let right_result = evaluate_(right.clone(), assignments)?;
            Ok(evaluate_infix(*operation, &left_result, &right_result))
        }
    }
}

fn evaluate_infix(operation: Operation, left: &Evaluated, right: &Evaluated) -> Evaluated {
    match (left, right) {
        (
            Evaluated::Primitive(Primitive::Integer(left)),
            Evaluated::Primitive(Primitive::Integer(right)),
        ) => Evaluated::Primitive(match operation {
            Operation::Add => Primitive::Integer(left + right),
            Operation::Subtract => Primitive::Integer(left - right),
            Operation::Multiply => Primitive::Integer(left * right),
        }),
    }
}
