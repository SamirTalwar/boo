//! Core data structures used by the evaluator.

use im::HashMap;

use boo_core::error::Result;
use boo_core::evaluation::{Evaluated, ExpressionReader};
use boo_core::expr::Function;
use boo_core::identifier::Identifier;
use boo_core::primitive::Primitive;

use crate::ast::Expr;
use crate::thunk::Thunk;

/// An interim evaluation result, with the same lifetime as the pool being
/// evaluated.
#[derive(Debug, Clone)]
pub enum EvaluationProgress<'a> {
    Primitive(Primitive),
    Closure {
        parameter: Identifier,
        body: Expr,
        bindings: Bindings<'a>,
    },
}

impl<'a> EvaluationProgress<'a> {
    /// Concludes evaluation.
    pub fn finish<Reader: ExpressionReader<Expr = Expr>>(self, reader: Reader) -> Evaluated {
        match self {
            Self::Primitive(primitive) => Evaluated::Primitive(primitive),
            Self::Closure {
                parameter,
                body,
                bindings: _,
            } => Evaluated::Function(Function {
                parameter,
                body: reader.to_core(body),
            }),
        }
    }
}

pub type UnevaluatedBinding<'a> = (Expr, Bindings<'a>);
pub type EvaluatedBinding<'a> = Result<EvaluationProgress<'a>>;
pub type Binding<'a> = Thunk<UnevaluatedBinding<'a>, EvaluatedBinding<'a>>;

/// The set of bindings in a given scope.
///
/// The variables bound in a specific scope are a mapping from an identifier to
/// the underlying expression. This expression is evaluated lazily, but only
/// once, using [`Thunk`].
#[derive(Debug, Clone)]
pub struct Bindings<'a>(HashMap<Identifier, Binding<'a>>);

impl<'a> Bindings<'a> {
    /// Constructs an empty set of bindings.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn read(
        &mut self,
        identifier: &Identifier,
    ) -> Option<&mut Thunk<UnevaluatedBinding<'a>, EvaluatedBinding<'a>>> {
        self.0.get_mut(identifier)
    }

    /// Adds a new binding to the set.
    pub fn with(
        &self,
        identifier: Identifier,
        expression: Expr,
        expression_bindings: Self,
    ) -> Self {
        Self(self.0.update(
            identifier,
            Thunk::unresolved((expression, expression_bindings)),
        ))
    }
}

impl<'a> Default for Bindings<'a> {
    fn default() -> Self {
        Self::new()
    }
}
