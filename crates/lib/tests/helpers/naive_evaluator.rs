use im::HashMap;

use boo::error::*;
use boo::identifier::*;
use boo::operation::*;
use boo::parser::ast::*;
use boo::primitive::*;

pub fn naively_evaluate(expr: Expr) -> Result<Expression> {
    evaluate_(expr, HashMap::new())
}

#[allow(clippy::boxed_local)]
fn evaluate_(expr: Expr, assignments: HashMap<Identifier, Expr>) -> Result<Expression> {
    match &expr.value {
        Expression::Primitive(_) => Ok(expr.value),
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

fn evaluate_infix(operation: Operation, left: &Expression, right: &Expression) -> Expression {
    match (left, right) {
        (
            Expression::Primitive(Primitive::Integer(left)),
            Expression::Primitive(Primitive::Integer(right)),
        ) => Expression::Primitive(match operation {
            Operation::Add => Primitive::Integer(left + right),
            Operation::Subtract => Primitive::Integer(left - right),
            Operation::Multiply => Primitive::Integer(left * right),
        }),
        _ => panic!(
            "evaluate_infix branch is not implemented for:\n  left:   {:?}\nright:  {:?}",
            left, right
        ),
    }
}
