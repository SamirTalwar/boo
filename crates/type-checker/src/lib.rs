//! Validates that a Boo AST is of a valid type, and annotates each expression
//! node with its type.

pub mod ast;

use std::rc::Rc;

use boo_core::ast::*;
use boo_core::types::*;

use ast::*;

/// Infers the type of a Boo expression.
///
/// If no type can be inferred, returns an error.
pub fn elaborate(expr: boo_parser::Expr) -> Expr {
    infer(expr)
}

fn infer(expr: boo_parser::Expr) -> Expr {
    let span = expr.annotation();
    match expr.expression() {
        Expression::Primitive(primitive) => Expr::new(
            (Type::Known(primitive.get_type().into()), span),
            Expression::Primitive(primitive),
        ),
        Expression::Identifier(_) => todo!("Identifier"),
        Expression::Assign(_) => todo!("Assign"),
        Expression::Function(_) => todo!("Function"),
        Expression::Apply(_) => todo!("Apply"),
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }) => {
            let elaborated_left = infer(left);
            let elaborated_right = infer(right);
            match (
                type_of(&elaborated_left).as_ref(),
                type_of(&elaborated_right).as_ref(),
            ) {
                (KnownType::Integer, KnownType::Integer) => Expr::new(
                    (Type::Known(KnownType::Integer.into()), span),
                    Expression::Infix(Infix {
                        operation,
                        left: elaborated_left,
                        right: elaborated_right,
                    }),
                ),
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
        let elaborated = elaborate(input);
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
        let elaborated = elaborate(input);
        assert_eq!(
            elaborated.get_type(),
            Type::Known(KnownType::Integer.into())
        );
    }
}
