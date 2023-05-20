use std::fmt::{Display, Result};

use crate::ast::*;

impl<Annotation, Value: Display> Display for Annotated<Annotation, Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        self.value.fmt(f)
    }
}

impl<Annotation> Display for Expression<Annotation> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
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
