//! Flattens an expression tree into a [`pool::Pool`].

pub mod ast;
pub mod builders;
pub mod pool;

use boo_core::ast::*;
use boo_core::span::*;

use ast::*;
use pool::pool_with;

/// Flattens an expression tree into a [`pool::Pool`].
pub fn pool_exprs<InputExpr>(ast: InputExpr) -> (ExprPool, Expr)
where
    InputExpr: ExpressionWrapper<Annotation = Span>,
{
    pool_with(|pool| add_expr(pool, ast))
}

/// Adds a single expression into the pool, recursively.
///
/// The leaf expressions will always be added before their parents, so that the
/// references are always valid.
pub fn add_expr<InputExpr>(pool: &mut ExprPool, expr: InputExpr) -> Expr
where
    InputExpr: ExpressionWrapper<Annotation = Span>,
{
    let annotation = expr.annotation();
    let expression = match expr.expression() {
        Expression::Primitive(x) => Expression::Primitive(x),
        Expression::Native(x) => Expression::Native(x),
        Expression::Identifier(x) => Expression::Identifier(x),
        Expression::Assign(Assign { name, value, inner }) => Expression::Assign(Assign {
            name,
            value: add_expr(pool, value),
            inner: add_expr(pool, inner),
        }),
        Expression::Function(Function { parameter, body }) => Expression::Function(Function {
            parameter,
            body: add_expr(pool, body),
        }),
        Expression::Apply(Apply { function, argument }) => Expression::Apply(Apply {
            function: add_expr(pool, function),
            argument: add_expr(pool, argument),
        }),
    };
    Expr::insert(pool, annotation, expression)
}
