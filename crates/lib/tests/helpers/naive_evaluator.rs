use im::HashMap;

use boo::error::*;
use boo::identifier::*;
use boo::operation::*;
use boo::parser::ast::*;
use boo::primitive::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evaluated {
    Primitive(Primitive),
    Function(Function, Bindings),
}

impl std::fmt::Display for Evaluated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluated::Primitive(x) => x.fmt(f),
            Evaluated::Function(x, _) => x.fmt(f),
        }
    }
}

pub fn naively_evaluate(expr: Expr) -> Result<Evaluated> {
    evaluate_(expr, Bindings(HashMap::new()))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bindings(HashMap<Identifier, (Expr, Bindings)>);

#[allow(clippy::boxed_local)]
fn evaluate_(expr: Expr, bindings: Bindings) -> Result<Evaluated> {
    match &expr.value {
        Expression::Primitive(value) => Ok(Evaluated::Primitive(value.clone())),
        Expression::Identifier(name) => match bindings.0.get(name) {
            Some((value, lookup_bindings)) => evaluate_(value.clone(), lookup_bindings.clone()),
            None => Err(Error::UnknownVariable {
                span: expr.span,
                name: name.to_string(),
            }),
        },
        Expression::Assign(Assign { name, value, inner }) => evaluate_(
            inner.clone(),
            Bindings(
                bindings
                    .clone()
                    .0
                    .update(name.clone(), (value.clone(), bindings)),
            ),
        ),
        Expression::Function(function) => Ok(Evaluated::Function(function.clone(), bindings)),
        Expression::Apply(Apply { function, argument }) => {
            let function_result = evaluate_(function.clone(), bindings)?;
            match function_result {
                Evaluated::Function(Function { parameter, body }, lookup_bindings) => evaluate_(
                    body,
                    Bindings(
                        lookup_bindings
                            .clone()
                            .0
                            .update(parameter, (argument.clone(), lookup_bindings)),
                    ),
                ),
                _ => Err(Error::InvalidFunctionApplication { span: expr.span }),
            }
        }
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }) => {
            let left_result = evaluate_(left.clone(), bindings.clone())?;
            let right_result = evaluate_(right.clone(), bindings)?;
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
        _ => panic!(
            "evaluate_infix branch is not implemented for:\n({}) {} ({})",
            left, operation, right
        ),
    }
}
