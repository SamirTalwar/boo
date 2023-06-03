//! Evaluates a parsed AST as simply as possible.
//!
//! This evaluator is not used by the interpreter. It is meant as an
//! implementation that is "so simple that there are obviously no deficiencies"
//! (to quote Tony Hoare). We then use it as a reference implementation to
//! validate that the real evaluator does the right thing when presented with an
//! arbitrary program.

use im::HashMap;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::identifier::*;
use boo_core::operation::*;
use boo_core::primitive::*;
use boo_parser::ast::*;

/// An evaluation result. This can be either a primitive value or a closure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evaluated {
    Primitive(Primitive),
    Closure(Function<Expr>, Bindings),
}

impl std::fmt::Display for Evaluated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluated::Primitive(x) => x.fmt(f),
            Evaluated::Closure(x, _) => x.fmt(f),
        }
    }
}

/// Evaluate a parsed AST as simply as possible.
pub fn naively_evaluate(expr: Expr) -> Result<Evaluated> {
    evaluate_(expr, Bindings(HashMap::new()))
}

/// The bound variables closed over by an expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bindings(HashMap<Identifier, (Expr, Bindings)>);

#[allow(clippy::boxed_local)]
fn evaluate_(expr: Expr, bindings: Bindings) -> Result<Evaluated> {
    let span = expr.annotation();
    match expr.expression() {
        Expression::Primitive(value) => Ok(Evaluated::Primitive(value)),
        Expression::Identifier(name) => match bindings.0.get(&name) {
            Some((value, lookup_bindings)) => evaluate_(value.clone(), lookup_bindings.clone()),
            None => Err(Error::UnknownVariable {
                span,
                name: name.to_string(),
            }),
        },
        Expression::Assign(Assign { name, value, inner }) => evaluate_(
            inner,
            Bindings(bindings.clone().0.update(name, (value, bindings))),
        ),
        Expression::Function(function) => Ok(Evaluated::Closure(function, bindings)),
        Expression::Apply(Apply { function, argument }) => {
            let function_result = evaluate_(function, bindings)?;
            match function_result {
                Evaluated::Closure(Function { parameter, body }, lookup_bindings) => evaluate_(
                    body,
                    Bindings(
                        lookup_bindings
                            .clone()
                            .0
                            .update(parameter, (argument, lookup_bindings)),
                    ),
                ),
                _ => Err(Error::InvalidFunctionApplication { span }),
            }
        }
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }) => {
            let left_result = evaluate_(left, bindings.clone())?;
            let right_result = evaluate_(right, bindings)?;
            Ok(evaluate_infix(operation, &left_result, &right_result))
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
