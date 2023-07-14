//! Flattens an expression tree into a [`pool::Pool`].

pub mod ast;
pub mod pool;

use boo_core::ast::ExpressionWrapper;
use boo_core::ast::*;

use ast::*;
use pool::pool_with;

/// Flattens an expression tree into a [`pool::Pool`].
pub fn pool_exprs(ast: boo_core::expr::Expr) -> (ExprPool, Expr) {
    pool_with(|pool| add_expr(pool, ast))
}

/// Adds a single expression into the pool, recursively.
///
/// The leaf expressions will always be added before their parents, so that the
/// references are always valid.
pub fn add_expr(pool: &mut ExprPool, expr: boo_core::expr::Expr) -> Expr {
    let span = expr.annotation();
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
    Expr::insert(pool, span, expression)
}
