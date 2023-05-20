use std::rc::Rc;

use crate::ast;

ast::expr! {
    wrapper = (Rc<_>),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Primitive { value } => value.fmt(f),
            Expression::Identifier { name } => name.fmt(f),
            Expression::Let { name, value, inner } => {
                write!(f, "let {} = ({}) in ({})", name, value, inner)
            }
            Expression::Infix {
                operation,
                left,
                right,
            } => write!(f, "({}) {} ({})", left, operation, right),
        }
    }
}
