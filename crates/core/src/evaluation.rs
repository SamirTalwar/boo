use crate::ast;
use crate::error::Result;
use crate::expr::Expr;
use crate::identifier::Identifier;
use crate::primitive::Primitive;
use crate::span::Spanned;

/// An evaluator knows how to evaluate expressions within a context.
///
/// Context can be added in the form of top-level bindings to other expressions.
pub trait Evaluator {
    /// Bind a new top-level expression.
    fn bind(&mut self, identifier: Identifier, expr: Expr) -> Result<()>;

    /// Evaluate the given expression.
    fn evaluate(&self, expr: Expr) -> Result<Evaluated>;
}

/// An evaluation result. This can be either a primitive value or a closure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evaluated {
    Primitive(Primitive),
    Function(ast::Function<Expr>),
}

impl std::fmt::Display for Evaluated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Evaluated::Primitive(x) => x.fmt(f),
            Evaluated::Function(x) => x.fmt(f),
        }
    }
}

/// Given an outer expression, consume it to retrieve the inner expression.
pub trait ExpressionReader: Copy {
    type Expr;
    type Target: AsRef<ast::Expression<Self::Expr>>;

    fn read(&self, expr: Self::Expr) -> Spanned<Self::Target>;

    // Recreates a core expression from the specified variant.
    fn to_core(&self, expr: Self::Expr) -> Expr
    where
        Self::Expr: Clone,
    {
        let Spanned {
            span,
            value: expression,
        } = self.read(expr);
        Expr::new(
            span,
            match expression.as_ref() {
                ast::Expression::Primitive(primitive) => {
                    ast::Expression::Primitive(primitive.clone())
                }
                ast::Expression::Native(native) => ast::Expression::Native(native.clone()),
                ast::Expression::Identifier(identifier) => {
                    ast::Expression::Identifier(identifier.clone())
                }
                ast::Expression::Function(ast::Function { parameter, body }) => {
                    ast::Expression::Function(ast::Function {
                        parameter: parameter.clone(),
                        body: self.to_core(body.clone()),
                    })
                }
                ast::Expression::Apply(ast::Apply { function, argument }) => {
                    ast::Expression::Apply(ast::Apply {
                        function: self.to_core(function.clone()),
                        argument: self.to_core(argument.clone()),
                    })
                }
                ast::Expression::Assign(ast::Assign { name, value, inner }) => {
                    ast::Expression::Assign(ast::Assign {
                        name: name.clone(),
                        value: self.to_core(value.clone()),
                        inner: self.to_core(inner.clone()),
                    })
                }
                ast::Expression::Match(ast::Match { value, patterns }) => {
                    ast::Expression::Match(ast::Match {
                        value: self.to_core(value.clone()),
                        patterns: patterns
                            .iter()
                            .map(|ast::PatternMatch { pattern, result }| ast::PatternMatch {
                                pattern: pattern.clone(),
                                result: self.to_core(result.clone()),
                            })
                            .collect(),
                    })
                }
                ast::Expression::Typed(ast::Typed { expression, typ }) => {
                    ast::Expression::Typed(ast::Typed {
                        expression: self.to_core(expression.clone()),
                        typ: typ.clone(),
                    })
                }
            },
        )
    }
}

impl<'a, T: ExpressionReader> ExpressionReader for &'a T {
    type Expr = <T as ExpressionReader>::Expr;
    type Target = <T as ExpressionReader>::Target;

    fn read(&self, expr: Self::Expr) -> Spanned<Self::Target> {
        <T as ExpressionReader>::read(self, expr)
    }
}
