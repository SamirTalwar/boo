use std::rc::Rc;

use crate::ast;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<Value> {
    pub span: Span,
    pub value: Value,
}

ast::expr!((Rc<Spanned<_>>));

impl<Value: std::fmt::Display> std::fmt::Display for Spanned<Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
