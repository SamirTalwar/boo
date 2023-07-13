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
    expr.transform(&mut |span, expression| Expr::insert(pool, span, expression))
}
