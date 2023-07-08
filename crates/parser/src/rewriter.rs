//! Rewrites the expression tree to avoid infix operations entirely.

use boo_core::ast::*;

use crate::Expr;

pub fn rewrite(expr: Expr) -> Expr {
    let annotation = expr.annotation();
    Expr::new(
        annotation,
        match expr.expression() {
            expression @ Expression::Primitive(_)
            | expression @ Expression::Native(_)
            | expression @ Expression::Identifier(_) => expression,
            Expression::Assign(Assign { name, value, inner }) => Expression::Assign(Assign {
                name,
                value: rewrite(value),
                inner: rewrite(inner),
            }),
            Expression::Function(Function { parameter, body }) => Expression::Function(Function {
                parameter,
                body: rewrite(body),
            }),
            Expression::Apply(Apply { function, argument }) => Expression::Apply(Apply {
                function: rewrite(function),
                argument: rewrite(argument),
            }),
            Expression::Infix(Infix {
                operation,
                left,
                right,
            }) => Expression::Apply(Apply {
                function: Expr::new(
                    annotation,
                    Expression::Apply(Apply {
                        function: Expr::new(
                            annotation,
                            Expression::Identifier(operation.identifier()),
                        ),
                        argument: rewrite(left),
                    }),
                ),
                argument: rewrite(right),
            }),
        },
    )
}
