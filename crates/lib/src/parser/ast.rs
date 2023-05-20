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
