//! An AST elaborated with type information at each node.

use boo_core::ast::{Expression, ExpressionWrapper};
use boo_core::span::{Span, Spanned};
use boo_core::types::{Type, Typed};

/// A wrapped expression where each child node is a reference to elsewhere in the pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr(Typed<Spanned<Box<Expression<Expr>>>>);

impl Expr {
    pub fn get_type(&self) -> Type {
        self.0.typ.clone()
    }
}

impl ExpressionWrapper for Expr {
    type Annotation = (Type, Span);

    fn new(annotation: Self::Annotation, value: Expression<Self>) -> Self {
        Self(Typed {
            typ: annotation.0,
            value: Spanned {
                span: annotation.1,
                value: value.into(),
            },
        })
    }

    fn new_unannotated(value: Expression<Self>) -> Self {
        Self(Typed {
            typ: Type::Unknown,
            value: Spanned {
                span: 0.into(),
                value: value.into(),
            },
        })
    }

    fn annotation(&self) -> Self::Annotation {
        (self.0.typ.clone(), self.0.value.span)
    }

    fn expression(self) -> Expression<Self> {
        *self.0.value.value
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
