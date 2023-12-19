use crate::error::{Error, Result};
use crate::expr;

pub fn verify(expr: &expr::Expr) -> Result<()> {
    match *expr.expression {
        expr::Expression::Primitive(_)
        | expr::Expression::Native(_)
        | expr::Expression::Identifier(_) => (),
        expr::Expression::Assign(expr::Assign {
            name: _,
            ref value,
            ref inner,
        }) => {
            verify(value)?;
            verify(inner)?;
        }
        expr::Expression::Function(expr::Function {
            parameter: _,
            ref body,
        }) => {
            verify(body)?;
        }
        expr::Expression::Match(expr::Match {
            ref value,
            ref patterns,
        }) => {
            match patterns.back().map(|p| &p.pattern) {
                Some(expr::Pattern::Anything) => Ok(()),
                _ => Err(Error::MatchWithoutBaseCase { span: expr.span }),
            }?;
            verify(value)?;
            for expr::PatternMatch { pattern: _, result } in patterns {
                verify(result)?;
            }
        }
        expr::Expression::Apply(expr::Apply {
            ref function,
            ref argument,
        }) => {
            verify(function)?;
            verify(argument)?;
        }
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::primitive::Primitive;

    use super::*;

    #[test]
    fn test_rejects_matches_without_a_base_case() {
        let expr = expr::Expr::new(
            Some((0..10).into()),
            expr::Expression::Match(expr::Match {
                value: expr::Expr::new(
                    Some((2..3).into()),
                    expr::Expression::Primitive(Primitive::Integer(1.into())),
                ),
                patterns: [expr::PatternMatch {
                    pattern: expr::Pattern::Primitive(Primitive::Integer(1.into())),
                    result: expr::Expr::new(
                        Some((7..8).into()),
                        expr::Expression::Primitive(Primitive::Integer(2.into())),
                    ),
                }]
                .into(),
            }),
        );

        let result = verify(&expr);

        assert_eq!(
            result,
            Err(Error::MatchWithoutBaseCase {
                span: Some((0..10).into())
            })
        );
    }
}
