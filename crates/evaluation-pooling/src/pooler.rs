//! Flattens an expression tree into a [`pool::Pool`].

use boo_core::ast::*;

use crate::ast::*;

/// Adds a single expression into the pool, recursively.
///
/// The leaf expressions will always be added before their parents, so that the
/// references are always valid.
pub fn add_expr(pool: &mut ExprPool, expr: boo_core::expr::Expr) -> Expr {
    let span = expr.span();
    let expression = match expr.take() {
        Expression::Primitive(x) => Expression::Primitive(x),
        Expression::Native(x) => Expression::Native(x),
        Expression::Identifier(x) => Expression::Identifier(x),
        Expression::Function(Function { parameter, body }) => Expression::Function(Function {
            parameter,
            body: add_expr(pool, body),
        }),
        Expression::Apply(Apply { function, argument }) => Expression::Apply(Apply {
            function: add_expr(pool, function),
            argument: add_expr(pool, argument),
        }),
        Expression::Assign(Assign { name, value, inner }) => Expression::Assign(Assign {
            name,
            value: add_expr(pool, value),
            inner: add_expr(pool, inner),
        }),
        Expression::Match(Match { value, patterns }) => Expression::Match(Match {
            value: add_expr(pool, value),
            patterns: patterns
                .into_iter()
                .map(|PatternMatch { pattern, result }| PatternMatch {
                    pattern,
                    result: add_expr(pool, result),
                })
                .collect(),
        }),
        Expression::Typed(Typed { expression, typ }) => Expression::Typed(Typed {
            expression: add_expr(pool, expression),
            typ,
        }),
    };
    Expr::insert(pool, span, expression)
}
