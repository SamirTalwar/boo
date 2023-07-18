//! Rewrites the expression tree to as a core AST.
//!
//! For now, this just rewrites infix operations as normal function application.

use boo_core::expr as core;

pub fn rewrite(expr: crate::Expr) -> core::Expr {
    core::Expr::new(
        Some(expr.span),
        match *expr.expression {
            crate::Expression::Primitive(x) => core::Expression::Primitive(x),
            crate::Expression::Identifier(x) => core::Expression::Identifier(x),
            crate::Expression::Assign(crate::Assign { name, value, inner }) => {
                core::Expression::Assign(core::Assign {
                    name,
                    value: rewrite(value),
                    inner: rewrite(inner),
                })
            }
            crate::Expression::Function(crate::Function { parameter, body }) => {
                core::Expression::Function(core::Function {
                    parameter,
                    body: rewrite(body),
                })
            }
            crate::Expression::Apply(crate::Apply { function, argument }) => {
                core::Expression::Apply(core::Apply {
                    function: rewrite(function),
                    argument: rewrite(argument),
                })
            }
            crate::Expression::Infix(crate::Infix {
                operation,
                left,
                right,
            }) => core::Expression::Apply(core::Apply {
                function: core::Expr::new(
                    Some(expr.span),
                    core::Expression::Apply(core::Apply {
                        function: core::Expr::new(
                            Some(expr.span),
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
