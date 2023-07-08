//! Built-in native functionality, required for evaluation of anything useful.

use std::sync::Arc;

use crate::ast::*;
use crate::identifier::Identifier;
use crate::native::Native;
use crate::primitive::{Integer, Primitive};

/// Prepares an expression for evaluation by assigning all built-ins.
///
/// This is very naive and probably quite slow; we can do better later.
pub fn prepare<E: ExpressionWrapper>(expr: E) -> E {
    let mut result = expr;
    for (name, builtin) in all().into_iter().rev() {
        result = E::new_unannotated(Expression::Assign(Assign {
            name,
            value: builtin,
            inner: result,
        }));
    }
    result
}

/// All the built-in expressions.
pub fn all<E: ExpressionWrapper>() -> Vec<(Identifier, E)> {
    vec![
        (Identifier::operator_from_str("+").unwrap(), builtin_add()),
        (
            Identifier::operator_from_str("-").unwrap(),
            builtin_subtract(),
        ),
        (
            Identifier::operator_from_str("*").unwrap(),
            builtin_multiply(),
        ),
        (Identifier::name_from_str("trace").unwrap(), builtin_trace()),
    ]
}

/// Implements addition, with the `+` operator.
fn builtin_add<E: ExpressionWrapper>() -> E {
    builtin_infix_math("+", |x, y| x + y)
}

/// Implements subtraction, with the `-` operator.
fn builtin_subtract<E: ExpressionWrapper>() -> E {
    builtin_infix_math("-", |x, y| x - y)
}

/// Implements multiplication, with the `*` operator.
fn builtin_multiply<E: ExpressionWrapper>() -> E {
    builtin_infix_math("*", |x, y| x * y)
}

/// Generic implementation of infix mathematical operations.
fn builtin_infix_math<E, Op>(name: &str, operate: Op) -> E
where
    E: ExpressionWrapper,
    Op: Fn(Integer, Integer) -> Integer + 'static,
{
    let parameter_left = Identifier::name_from_str("left").unwrap();
    let parameter_right = Identifier::name_from_str("right").unwrap();
    E::new_unannotated(Expression::Function(Function {
        parameter: parameter_left.clone(),
        body: E::new_unannotated(Expression::Function(Function {
            parameter: parameter_right.clone(),
            body: E::new_unannotated(Expression::Native(Native {
                unique_name: Identifier::operator_from_str(name).unwrap(),
                implementation: Arc::new(move |context| {
                    let left = context.lookup_value(&parameter_left)?;
                    let right = context.lookup_value(&parameter_right)?;
                    match (left, right) {
                        (Primitive::Integer(left), Primitive::Integer(right)) => {
                            Ok(Primitive::Integer(operate(left, right)))
                        }
                    }
                }),
            })),
        })),
    }))
}

/// A "trace" function, which prints the computed value.
fn builtin_trace<E: ExpressionWrapper>() -> E {
    let parameter = Identifier::name_from_str("param").unwrap();
    E::new_unannotated(Expression::Function(Function {
        parameter: parameter.clone(),
        body: E::new_unannotated(Expression::Native(Native {
            unique_name: Identifier::name_from_str("trace").unwrap(),
            implementation: Arc::new(move |context| {
                let value = context.lookup_value(&parameter)?;
                eprintln!("trace: {}", value);
                Ok(value)
            }),
        })),
    }))
}
