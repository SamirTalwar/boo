//! Validates that a Boo AST is of a valid type, and annotates each expression
//! node with its type.

pub mod ast;

use std::rc::Rc;

use im::HashMap;

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::identifier::*;
use boo_core::types::*;

use ast::*;

type Bindings = HashMap<Identifier, Rc<KnownType>>;

/// Infers the type of a Boo expression.
///
/// If no type can be inferred, returns an error.
pub fn elaborate(expr: boo_parser::Expr) -> Result<Expr> {
    infer(expr, HashMap::new())
}

fn infer(expr: boo_parser::Expr, bindings: Bindings) -> Result<Expr> {
    let span = expr.annotation();
    match expr.expression() {
        Expression::Primitive(primitive) => Ok(Expr::new(
            (Type::Known(primitive.get_type().into()), span),
            Expression::Primitive(primitive),
        )),
        Expression::Identifier(name) => {
            let name_type = bindings
                .get(&name)
                .map(Ok)
                .unwrap_or(Err(Error::UnknownVariable {
                    span,
                    name: name.to_string(),
                }))?;
            Ok(Expr::new(
                (Type::Known(name_type.clone()), span),
                Expression::Identifier(name),
            ))
        }
        Expression::Assign(Assign { name, value, inner }) => {
            let elaborated_value = infer(value, bindings.clone())?;
            let elaborated_inner = infer(
                inner,
                bindings.update(name.clone(), type_of(&elaborated_value)),
            )?;
            Ok(Expr::new(
                (Type::Known(type_of(&elaborated_inner)), span),
                Expression::Assign(Assign {
                    name,
                    value: elaborated_value,
                    inner: elaborated_inner,
                }),
            ))
        }
        Expression::Function(Function { parameter, body }) => todo!("Function"),
        Expression::Apply(Apply { function, argument }) => todo!("Apply"),
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }) => {
            let elaborated_left = infer(left, bindings.clone())?;
            let elaborated_right = infer(right, bindings)?;
            match (
                type_of(&elaborated_left).as_ref(),
                type_of(&elaborated_right).as_ref(),
            ) {
                (KnownType::Integer, KnownType::Integer) => Ok(Expr::new(
                    (Type::Known(KnownType::Integer.into()), span),
                    Expression::Infix(Infix {
                        operation,
                        left: elaborated_left,
                        right: elaborated_right,
                    }),
                )),
                _ => todo!("Infix"),
            }
        }
    }
}

fn type_of(expr: &Expr) -> Rc<KnownType> {
    match expr.annotation().0 {
        Type::Known(known) => known,
        Type::Unknown => panic!("Failed to elaborate the type of: {}", expr),
    }
}

#[cfg(test)]
mod tests {
    use boo_core::ast::builders;
    use boo_core::operation::*;
    use boo_core::primitive::*;

    use super::*;

    #[test]
    fn elaborates_an_integer() {
        let input = builders::primitive_integer(0, Integer::Small(7));
        let elaborated = elaborate(input).unwrap();
        assert_eq!(
            elaborated.get_type(),
            Type::Known(KnownType::Integer.into())
        );
    }

    #[test]
    fn elaborates_a_pointless_assigment() {
        let input = builders::assign(
            0,
            Identifier::new("name".to_string()).unwrap(),
            builders::primitive_integer(0, Integer::Small(2)),
            builders::primitive_integer(0, Integer::Small(3)),
        );
        let elaborated = elaborate(input).unwrap();
        assert_eq!(
            elaborated.get_type(),
            Type::Known(KnownType::Integer.into())
        );
    }

    #[test]
    fn elaborates_a_meaningful_assignment() {
        let variable_name = Identifier::new("name".to_string()).unwrap();
        let input = builders::assign(
            0,
            variable_name.clone(),
            builders::primitive_integer(0, Integer::Small(4)),
            builders::identifier(0, variable_name),
        );
        let elaborated = elaborate(input).unwrap();
        assert_eq!(
            elaborated.get_type(),
            Type::Known(KnownType::Integer.into())
        );
    }

    #[test]
    fn elaborates_an_infix_operation_on_integers() {
        let input = builders::infix(
            0,
            Operation::Multiply,
            builders::primitive_integer(0, Integer::Small(7)),
            builders::primitive_integer(0, Integer::Small(6)),
        );
        let elaborated = elaborate(input).unwrap();
        assert_eq!(
            elaborated.get_type(),
            Type::Known(KnownType::Integer.into())
        );
    }

    #[test]
    #[ignore = "we do not support non-integer values yet"]
    fn fails_to_elaborate_an_infix_operation_on_a_non_integer_value() {
        let input = builders::infix(
            0,
            Operation::Multiply,
            builders::primitive_integer(0, Integer::Small(7)),
            builders::function(
                0,
                Identifier::new("name".to_string()).unwrap(),
                builders::primitive_integer(0, Integer::Small(1)),
            ),
        );
        let elaborated = elaborate(input).unwrap();
        assert_eq!(
            elaborated.get_type(),
            Type::Known(KnownType::Integer.into())
        );
    }
}
