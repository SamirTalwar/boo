//! Flattens an expression tree into a [`pool::Pool`].

use boo_core::ast::*;

use crate::ast::*;

/// Adds a single expression into the pool, recursively.
///
/// The leaf expressions will always be added before their parents, so that the
/// references are always valid.
pub fn add_expr(pool: &mut ExprPool, expr: boo_core::expr::Expr) -> Expr {
    let expression = match *expr.expression {
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
    Expr::insert(pool, expr.span, expression)
}

// Recreates a core expression from the flattened variant.
pub fn unpool_expr(pool: &ExprPool, expr: Expr) -> boo_core::expr::Expr {
    let Inner { span, expression } = expr.read_from(pool);
    match expression {
        Expression::Primitive(primitive) => {
            boo_core::expr::Expr::new(*span, Expression::Primitive(primitive.clone()))
        }
        Expression::Native(native) => {
            boo_core::expr::Expr::new(*span, Expression::Native(native.clone()))
        }
        Expression::Identifier(identifier) => {
            boo_core::expr::Expr::new(*span, Expression::Identifier(identifier.clone()))
        }
        Expression::Assign(Assign { name, value, inner }) => boo_core::expr::Expr::new(
            *span,
            Expression::Assign(Assign {
                name: name.clone(),
                value: unpool_expr(pool, *value),
                inner: unpool_expr(pool, *inner),
            }),
        ),
        Expression::Function(Function { parameter, body }) => boo_core::expr::Expr::new(
            *span,
            Expression::Function(Function {
                parameter: parameter.clone(),
                body: unpool_expr(pool, *body),
            }),
        ),
        Expression::Apply(Apply { function, argument }) => boo_core::expr::Expr::new(
            *span,
            Expression::Apply(Apply {
                function: unpool_expr(pool, *function),
                argument: unpool_expr(pool, *argument),
            }),
        ),
    }
}
