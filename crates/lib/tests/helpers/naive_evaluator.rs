use im::HashMap;

use boo::error::*;
use boo::identifier::*;
use boo::operation::*;
use boo::parser::ast::*;
use boo::primitive::*;

pub fn naively_evaluate(expr: Expr) -> Result<Primitive> {
    evaluate_(expr, HashMap::new())
}

#[allow(clippy::boxed_local)]
fn evaluate_(expr: Expr, assignments: HashMap<Identifier, Expr>) -> Result<Primitive> {
    match &expr.value {
        Expression::Primitive { value } => Ok(value.clone()),
        Expression::Identifier { name } => match assignments.get(name) {
            Some(value) => evaluate_(value.clone(), assignments),
            None => Err(Error::UnknownVariable {
                span: expr.span,
                name: name.to_string(),
            }),
        },
        Expression::Assign { name, value, inner } => evaluate_(
            inner.clone(),
            assignments.update(name.clone(), value.clone()),
        ),
        Expression::Infix {
            operation,
            left,
            right,
        } => match (&left.value, &right.value) {
            (
                Expression::Primitive { value: left_value },
                Expression::Primitive { value: right_value },
            ) => Ok(evaluate_infix(*operation, left_value, right_value)),
            _ => {
                let left_result = evaluate_(left.clone(), assignments.clone())?;
                let right_result = evaluate_(right.clone(), assignments)?;
                Ok(evaluate_infix(*operation, &left_result, &right_result))
            }
        },
    }
}

fn evaluate_infix(operation: Operation, left: &Primitive, right: &Primitive) -> Primitive {
    match (left, right) {
        (Primitive::Integer(left), Primitive::Integer(right)) => match operation {
            Operation::Add => Primitive::Integer(left + right),
            Operation::Subtract => Primitive::Integer(left - right),
            Operation::Multiply => Primitive::Integer(left * right),
        },
    }
}
