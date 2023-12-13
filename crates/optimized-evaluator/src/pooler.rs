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
        Expression::Match(Match { value, patterns }) => Expression::Match(Match {
            value: add_expr(pool, value),
            patterns: add_pattern_match(pool, patterns),
        }),
        Expression::Apply(Apply { function, argument }) => Expression::Apply(Apply {
            function: add_expr(pool, function),
            argument: add_expr(pool, argument),
        }),
    };
    Expr::insert(pool, expr.span, expression)
}

/// Adds a pattern match into the pool, recursively.
fn add_pattern_match(
    pool: &mut ExprPool,
    pattern_match: boo_core::expr::PatternMatch<boo_core::expr::Expr>,
) -> PatternMatch<Expr> {
    match pattern_match {
        PatternMatch::Anything { result } => PatternMatch::Anything {
            result: add_expr(pool, result),
        },
        PatternMatch::Primitive {
            pattern,
            matched,
            not_matched,
        } => PatternMatch::Primitive {
            pattern,
            matched: add_expr(pool, matched),
            not_matched: Box::new(add_pattern_match(pool, *not_matched)),
        },
    }
}

// Recreates a core expression from the flattened variant.
pub fn unpool_expr(pool: &ExprPool, expr: Expr) -> boo_core::expr::Expr {
    let Inner { span, expression } = expr.read_from(pool);
    boo_core::expr::Expr::new(
        *span,
        match expression {
            Expression::Primitive(primitive) => Expression::Primitive(primitive.clone()),
            Expression::Native(native) => Expression::Native(native.clone()),
            Expression::Identifier(identifier) => Expression::Identifier(identifier.clone()),
            Expression::Assign(Assign { name, value, inner }) => Expression::Assign(Assign {
                name: name.clone(),
                value: unpool_expr(pool, *value),
                inner: unpool_expr(pool, *inner),
            }),
            Expression::Function(Function { parameter, body }) => Expression::Function(Function {
                parameter: parameter.clone(),
                body: unpool_expr(pool, *body),
            }),
            Expression::Match(Match { value, patterns }) => Expression::Match(Match {
                value: unpool_expr(pool, *value),
                patterns: unpool_pattern_match(pool, patterns),
            }),
            Expression::Apply(Apply { function, argument }) => Expression::Apply(Apply {
                function: unpool_expr(pool, *function),
                argument: unpool_expr(pool, *argument),
            }),
        },
    )
}

// Recreates a core pattern match from the flattened variant.
fn unpool_pattern_match(
    pool: &ExprPool,
    pattern_match: &PatternMatch<Expr>,
) -> boo_core::expr::PatternMatch<boo_core::expr::Expr> {
    match pattern_match {
        PatternMatch::Anything { result } => PatternMatch::Anything {
            result: unpool_expr(pool, *result),
        },
        PatternMatch::Primitive {
            pattern,
            matched,
            not_matched,
        } => PatternMatch::Primitive {
            pattern: pattern.clone(),
            matched: unpool_expr(pool, *matched),
            not_matched: Box::new(unpool_pattern_match(pool, not_matched)),
        },
    }
}
