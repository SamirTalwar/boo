//! Rewrites the expression tree to as a core AST.
//!
//! For now, this just rewrites infix operations as normal function application.

use boo_core::expr as core;

pub fn rewrite(expr: crate::Expr) -> core::Expr {
    let wrap = { |expression| core::Expr::new(Some(expr.span), expression) };
    match *expr.expression {
        crate::Expression::Primitive(x) => wrap(core::Expression::Primitive(x)),
        crate::Expression::Identifier(x) => wrap(core::Expression::Identifier(x)),
        crate::Expression::Assign(crate::Assign { name, value, inner }) => {
            wrap(core::Expression::Assign(core::Assign {
                name,
                value: rewrite(value),
                inner: rewrite(inner),
            }))
        }
        crate::Expression::Function(crate::Function { parameters, body }) => {
            let mut expr = rewrite(body);
            for parameter in parameters.into_iter().rev() {
                expr = wrap(core::Expression::Function(core::Function {
                    parameter,
                    body: expr,
                }));
            }
            expr
        }
        crate::Expression::Apply(crate::Apply { function, argument }) => {
            wrap(core::Expression::Apply(core::Apply {
                function: rewrite(function),
                argument: rewrite(argument),
            }))
        }
        crate::Expression::Infix(crate::Infix {
            operation,
            left,
            right,
        }) => wrap(core::Expression::Apply(core::Apply {
            function: wrap(core::Expression::Apply(core::Apply {
                function: wrap(core::Expression::Identifier(operation.identifier())),
                argument: rewrite(left),
            })),
            argument: rewrite(right),
        })),
    }
}
