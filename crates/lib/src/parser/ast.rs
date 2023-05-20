use std::rc::Rc;

use crate::ast;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotated<Annotation, Value> {
    pub annotation: Annotation,
    pub value: Value,
}

ast::expr! {
    wrapper = (Rc<Annotated<Annotation, _>>),
    parameters = Annotation,
}

impl<Annotation, Value: std::fmt::Display> std::fmt::Display for Annotated<Annotation, Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<Annotation> std::fmt::Display for Expression<Annotation> {
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
