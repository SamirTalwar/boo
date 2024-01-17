//! Rewrites the expression tree to as a core AST.
//!
//! For now, this just rewrites infix operations as normal function application.

use boo_core::error::Result;
use boo_core::expr as core;

pub fn rewrite(expr: crate::Expr) -> Result<core::Expr> {
    let wrap = { |expression| core::Expr::new(Some(expr.span), expression) };
    Ok(match *expr.expression {
        crate::Expression::Primitive(x) => wrap(core::Expression::Primitive(x)),
        crate::Expression::Identifier(x) => wrap(core::Expression::Identifier(x)),
        crate::Expression::Function(crate::Function { parameters, body }) => {
            let mut expr = rewrite(body)?;
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
                function: rewrite(function)?,
                argument: rewrite(argument)?,
            }))
        }
        crate::Expression::Assign(crate::Assign { name, value, inner }) => {
            wrap(core::Expression::Assign(core::Assign {
                name,
                value: rewrite(value)?,
                inner: rewrite(inner)?,
            }))
        }
        crate::Expression::Match(crate::Match { value, patterns }) => {
            wrap(core::Expression::Match(core::Match {
                value: rewrite(value)?,
                patterns: patterns
                    .into_iter()
                    .map(
                        |crate::PatternMatch { pattern, result }| -> Result<core::PatternMatch<_>> {
                            let rewritten_pattern = match pattern {
                                crate::Pattern::Anything => core::Pattern::Anything,
                                crate::Pattern::Primitive(x) => core::Pattern::Primitive(x),
                            };
                            Ok(core::PatternMatch {
                                pattern: rewritten_pattern,
                                result: rewrite(result)?,
                            })
                        },
                    )
                    .collect::<Result<_>>()?,
            }))
        }
        crate::Expression::Infix(crate::Infix {
            operation,
            left,
            right,
        }) => wrap(core::Expression::Apply(core::Apply {
            function: wrap(core::Expression::Apply(core::Apply {
                function: wrap(core::Expression::Identifier(operation.identifier())),
                argument: rewrite(left)?,
            })),
            argument: rewrite(right)?,
        })),
        crate::Expression::Typed(crate::Typed { expression, typ }) => {
            wrap(core::Expression::Typed(core::Typed {
                expression: rewrite(expression)?,
                typ,
            }))
        }
    })
}

#[cfg(test)]
mod tests {
    use boo_core::identifier::Identifier;
    use boo_core::primitive::Primitive;

    use super::*;

    #[test]
    fn test_infix_expressions_are_converted_to_nested_function_applications() -> anyhow::Result<()>
    {
        let a = Primitive::Integer(3.into());
        let b = Primitive::Integer(5.into());
        let expression = crate::Expr::new(
            (0..5).into(),
            crate::Expression::Infix(crate::Infix {
                operation: crate::Operation::Add,
                left: crate::Expr::new((0..1).into(), crate::Expression::Primitive(a.clone())),
                right: crate::Expr::new((4..5).into(), crate::Expression::Primitive(b.clone())),
            }),
        );

        let expected = core::Expr::new(
            Some((0..5).into()),
            core::Expression::Apply(core::Apply {
                function: core::Expr::new(
                    Some((0..5).into()),
                    core::Expression::Apply(core::Apply {
                        function: core::Expr::new(
                            Some((0..5).into()),
                            core::Expression::Identifier(Identifier::operator_from_str("+")?),
                        ),
                        argument: core::Expr::new(
                            Some((0..1).into()),
                            core::Expression::Primitive(a),
                        ),
                    }),
                ),
                argument: core::Expr::new(Some((4..5).into()), core::Expression::Primitive(b)),
            }),
        );

        let actual = rewrite(expression)?;

        assert_eq!(actual, expected);
        Ok(())
    }
}
