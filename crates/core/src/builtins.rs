//! Built-in native functionality, required for evaluation of anything useful.

use std::rc::Rc;

use lazy_static::lazy_static;

use crate::ast::*;
use crate::error::Result;
use crate::evaluation::EvaluationContext;
use crate::expr::Expr;
use crate::identifier::Identifier;
use crate::native::Native;
use crate::primitive::{Integer, Primitive};
use crate::types::{Monotype, Polytype, Type, TypeVariable};

lazy_static! {
    static ref NAME_ADD: Identifier = Identifier::operator_from_str("+").unwrap();
    static ref NAME_SUBTRACT: Identifier = Identifier::operator_from_str("-").unwrap();
    static ref NAME_MULTIPLY: Identifier = Identifier::operator_from_str("*").unwrap();
    static ref NAME_TRACE: Identifier = Identifier::name_from_str("trace").unwrap();
}

/// Prepares an [EvaluationContext] by assigning all built-ins.
pub fn prepare(context: &mut impl EvaluationContext) -> Result<()> {
    for builtin in all().into_iter().rev() {
        context.bind(builtin.name.clone(), builtin.implementation)?;
    }
    Ok(())
}

/// Prepares an evaluator by assigning all built-ins.
pub fn types() -> impl Iterator<Item = (&'static Identifier, Polytype)> {
    all()
        .into_iter()
        .map(|builtin| (builtin.name, builtin.assumed_type))
}

struct Builtin {
    name: &'static Identifier,
    assumed_type: Polytype,
    implementation: Expr,
}

/// All the built-in expressions.
fn all() -> Vec<Builtin> {
    vec![
        Builtin {
            name: &NAME_ADD,
            assumed_type: Polytype::unquantified(
                Type::Function {
                    parameter: Type::Integer.into(),
                    body: Type::Function {
                        parameter: Type::Integer.into(),
                        body: Type::Integer.into(),
                    }
                    .into(),
                }
                .into(),
            ),
            implementation: builtin_add(),
        },
        Builtin {
            name: &NAME_SUBTRACT,
            assumed_type: Polytype::unquantified(
                Type::Function {
                    parameter: Type::Integer.into(),
                    body: Type::Function {
                        parameter: Type::Integer.into(),
                        body: Type::Integer.into(),
                    }
                    .into(),
                }
                .into(),
            ),
            implementation: builtin_subtract(),
        },
        Builtin {
            name: &NAME_MULTIPLY,
            assumed_type: Polytype::unquantified(
                Type::Function {
                    parameter: Type::Integer.into(),
                    body: Type::Function {
                        parameter: Type::Integer.into(),
                        body: Type::Integer.into(),
                    }
                    .into(),
                }
                .into(),
            ),
            implementation: builtin_multiply(),
        },
        Builtin {
            name: &NAME_TRACE,
            assumed_type: {
                let variable = TypeVariable::new_from_str("a");
                let variable_ref: Monotype = Type::Variable(variable.clone()).into();
                Polytype {
                    quantifiers: vec![variable],
                    mono: Type::Function {
                        parameter: variable_ref.clone(),
                        body: variable_ref,
                    }
                    .into(),
                }
            },
            implementation: builtin_trace(),
        },
    ]
}

/// Implements addition, with the `+` operator.
fn builtin_add() -> Expr {
    builtin_infix_math("+", |x, y| x + y)
}

/// Implements subtraction, with the `-` operator.
fn builtin_subtract() -> Expr {
    builtin_infix_math("-", |x, y| x - y)
}

/// Implements multiplication, with the `*` operator.
fn builtin_multiply() -> Expr {
    builtin_infix_math("*", |x, y| x * y)
}

/// Generic implementation of infix mathematical operations.
fn builtin_infix_math<Op>(name: &str, operate: Op) -> Expr
where
    Op: Fn(Integer, Integer) -> Integer + 'static,
{
    let parameter_left = Identifier::name_from_str("left").unwrap();
    let parameter_right = Identifier::name_from_str("right").unwrap();
    Expr::new(
        None,
        Expression::Function(Function {
            parameter: parameter_left.clone(),
            body: Expr::new(
                None,
                Expression::Function(Function {
                    parameter: parameter_right.clone(),
                    body: Expr::new(
                        None,
                        Expression::Native(Native {
                            unique_name: Identifier::operator_from_str(name).unwrap(),
                            implementation: Rc::new(move |context| {
                                let left = context.lookup_value(&parameter_left)?;
                                let right = context.lookup_value(&parameter_right)?;
                                match (left, right) {
                                    (Primitive::Integer(left), Primitive::Integer(right)) => {
                                        Ok(Primitive::Integer(operate(left, right)))
                                    }
                                }
                            }),
                        }),
                    ),
                }),
            ),
        }),
    )
}

/// A "trace" function, which prints the computed value.
fn builtin_trace() -> Expr {
    let parameter = Identifier::name_from_str("param").unwrap();
    Expr::new(
        None,
        Expression::Function(Function {
            parameter: parameter.clone(),
            body: Expr::new(
                None,
                Expression::Native(Native {
                    unique_name: Identifier::name_from_str("trace").unwrap(),
                    implementation: Rc::new(move |context| {
                        let value = context.lookup_value(&parameter)?;
                        eprintln!("trace: {}", value);
                        Ok(value)
                    }),
                }),
            ),
        }),
    )
}
