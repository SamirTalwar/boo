//! Rewrites the expression tree to avoid infix operations entirely.

use boo_core::ast::ExpressionWrapper;
use boo_core::expr as core;
use boo_core::span::HasSpan;

use boo_language as language;

pub fn rewrite(expr: language::Expr) -> core::Expr {
    let span = expr.span();
    core::Expr::new(
        span,
        match expr.expression() {
            language::Expression::Primitive(x) => core::Expression::Primitive(x),
            language::Expression::Identifier(x) => core::Expression::Identifier(x),
            language::Expression::Assign(language::Assign { name, value, inner }) => {
                core::Expression::Assign(core::Assign {
                    name,
                    value: rewrite(value),
                    inner: rewrite(inner),
                })
            }
            language::Expression::Function(language::Function { parameter, body }) => {
                core::Expression::Function(core::Function {
                    parameter,
                    body: rewrite(body),
                })
            }
            language::Expression::Apply(language::Apply { function, argument }) => {
                core::Expression::Apply(core::Apply {
                    function: rewrite(function),
                    argument: rewrite(argument),
                })
            }
            language::Expression::Infix(language::Infix {
                operation,
                left,
                right,
            }) => core::Expression::Apply(core::Apply {
                function: core::Expr::new(
                    span,
                    core::Expression::Apply(core::Apply {
                        function: core::Expr::new(
                            span,
                            core::Expression::Identifier(operation.identifier()),
                        ),
                        argument: rewrite(left),
                    }),
                ),
                argument: rewrite(right),
            }),
        },
    )
}
