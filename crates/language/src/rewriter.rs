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
        crate::Expression::Match(crate::Match {
            value,
            mut patterns,
        }) => {
            let crate::PatternMatch {
                pattern: base_pattern,
                result: base_result,
            } = patterns.pop().unwrap();
            let mut rewritten_patterns = match base_pattern {
                crate::Pattern::Anything => core::PatternMatch::Anything {
                    result: rewrite(base_result),
                },
                _ => panic!("FATAL: Encountered a match expression without a base case."),
            };
            for crate::PatternMatch { pattern, result } in patterns.into_iter().rev() {
                match pattern {
                    crate::Pattern::Primitive(primitive) => {
                        rewritten_patterns = core::PatternMatch::Primitive {
                            pattern: primitive,
                            matched: rewrite(result),
                            not_matched: Box::new(rewritten_patterns),
                        };
                    }
                    crate::Pattern::Anything => {
                        rewritten_patterns = core::PatternMatch::Anything {
                            result: rewrite(result),
                        };
                    }
                }
            }
            wrap(core::Expression::Match(core::Match {
                value: rewrite(value),
                patterns: rewritten_patterns,
            }))
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
