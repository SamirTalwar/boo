pub mod ast;
pub mod builders;
pub mod pool;

use crate::parser;
use crate::span::Spanned;

use ast::*;
use pool::pool_with;

pub fn pool_exprs(ast: &parser::ast::Expr) -> ExprPool {
    pool_with(|pool| {
        add_expr(pool, ast);
    })
}

pub fn add_expr(pool: &mut ExprPool, expr: &parser::ast::Expr) -> ExprRef {
    match &expr.value {
        parser::ast::Expression::Primitive { value } => pool.add(Spanned {
            span: expr.span,
            value: Expression::Primitive {
                value: value.clone(),
            },
        }),
        parser::ast::Expression::Identifier { name } => pool.add(Spanned {
            span: expr.span,
            value: Expression::Identifier { name: name.clone() },
        }),
        parser::ast::Expression::Let { name, value, inner } => {
            let value_ref = add_expr(pool, value);
            let inner_ref = add_expr(pool, inner);
            pool.add(Spanned {
                span: expr.span,
                value: Expression::Let {
                    name: name.clone(),
                    value: value_ref,
                    inner: inner_ref,
                },
            })
        }
        parser::ast::Expression::Infix {
            operation,
            left,
            right,
        } => {
            let left_ref = add_expr(pool, left);
            let right_ref = add_expr(pool, right);
            pool.add(Spanned {
                span: expr.span,
                value: Expression::Infix {
                    operation: *operation,
                    left: left_ref,
                    right: right_ref,
                },
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::identifier::*;
    use crate::operation::*;
    use crate::parser;
    use crate::primitive::*;
    use crate::proptest_helpers::check;

    use super::*;

    #[test]
    fn test_single_primitive() {
        check(&Integer::arbitrary(), |value| {
            let input = parser::builders::primitive_integer(0..0, value.clone());
            let expected = pool_with(|pool| {
                builders::primitive_integer(pool, value.clone());
            });

            let actual = pool_exprs(&input);

            prop_assert_eq!(actual, expected);
            Ok(())
        })
    }

    #[test]
    fn test_single_identifier() {
        check(&Identifier::arbitrary(), |name| {
            let input = parser::builders::identifier(0..0, name.clone());
            let expected = pool_with(|pool| {
                builders::identifier(pool, name.clone());
            });

            let actual = pool_exprs(&input);

            prop_assert_eq!(actual, expected);
            Ok(())
        })
    }

    #[test]
    fn test_assignment() {
        check(
            &(
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(name, value, inner)| {
                let input = parser::builders::assign(
                    0..0,
                    name.clone(),
                    parser::builders::primitive_integer(0..0, value.clone()),
                    parser::builders::primitive_integer(0..0, inner.clone()),
                );
                let expected = pool_with(|pool| {
                    let value_ref = builders::primitive_integer(pool, value.clone());
                    let inner_ref = builders::primitive_integer(pool, inner.clone());
                    builders::assign(pool, name.clone(), value_ref, inner_ref);
                });

                let actual = pool_exprs(&input);

                prop_assert_eq!(actual, expected);
                Ok(())
            },
        )
    }

    #[test]
    fn test_infix_operations() {
        check(
            &(
                any::<Operation>(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(operation, left, right)| {
                let input = parser::builders::infix(
                    0..0,
                    operation,
                    parser::builders::primitive_integer(0..0, left.clone()),
                    parser::builders::primitive_integer(0..0, right.clone()),
                );
                let expected = pool_with(|pool| {
                    let left_ref = builders::primitive_integer(pool, left.clone());
                    let right_ref = builders::primitive_integer(pool, right.clone());
                    builders::infix(pool, operation, left_ref, right_ref);
                });

                let actual = pool_exprs(&input);

                prop_assert_eq!(actual, expected);
                Ok(())
            },
        )
    }
}
