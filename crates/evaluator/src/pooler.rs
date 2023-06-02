pub mod ast;
pub mod builders;
pub mod pool;

use boo_core::ast::*;
use boo_parser as parser;

use ast::*;
use pool::pool_with;

pub fn pool_exprs(ast: parser::Expr) -> ExprPool {
    pool_with(|pool| {
        add_expr(pool, ast);
    })
}

pub fn add_expr(pool: &mut ExprPool, expr: parser::Expr) -> Expr {
    expr.map(&mut |span, expression| Expr::insert(pool, span, expression))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use boo_test_helpers::proptest::*;

    use boo_core::ast;
    use boo_core::identifier::*;
    use boo_core::operation::*;
    use boo_core::primitive::*;

    use super::*;

    #[test]
    fn test_single_primitive() {
        check(&Integer::arbitrary(), |value| {
            let input = ast::builders::primitive_integer(0..0, value.clone());
            let expected = pool_with(|pool| {
                builders::primitive_integer(pool, value.clone());
            });

            let actual = pool_exprs(input);

            prop_assert_eq!(actual, expected);
            Ok(())
        })
    }

    #[test]
    fn test_single_identifier() {
        check(&Identifier::arbitrary(), |name| {
            let input = ast::builders::identifier(0..0, name.clone());
            let expected = pool_with(|pool| {
                builders::identifier(pool, name.clone());
            });

            let actual = pool_exprs(input);

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
                let input = ast::builders::assign(
                    0..0,
                    name.clone(),
                    ast::builders::primitive_integer(0..0, value.clone()),
                    ast::builders::primitive_integer(0..0, inner.clone()),
                );
                let expected = pool_with(|pool| {
                    let value_ref = builders::primitive_integer(pool, value.clone());
                    let inner_ref = builders::primitive_integer(pool, inner.clone());
                    builders::assign(pool, name.clone(), value_ref, inner_ref);
                });

                let actual = pool_exprs(input);

                prop_assert_eq!(actual, expected);
                Ok(())
            },
        )
    }

    #[test]
    fn test_functions() {
        check(
            &(Identifier::arbitrary(), Integer::arbitrary()),
            |(parameter, modifier)| {
                let input = ast::builders::function(
                    0..0,
                    parameter.clone(),
                    ast::builders::infix(
                        0..0,
                        Operation::Add,
                        ast::builders::identifier(0..0, parameter.clone()),
                        ast::builders::primitive_integer(0..0, modifier.clone()),
                    ),
                );
                let expected = pool_with(|pool| {
                    let left_ref = builders::identifier(pool, parameter.clone());
                    let right_ref = builders::primitive_integer(pool, modifier.clone());
                    let add_ref = builders::infix(pool, Operation::Add, left_ref, right_ref);
                    builders::function(pool, parameter.clone(), add_ref);
                });

                let actual = pool_exprs(input);

                prop_assert_eq!(actual, expected);
                Ok(())
            },
        )
    }

    #[test]
    fn test_function_application() {
        check(
            &(
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(parameter, modifier, value)| {
                let input = ast::builders::apply(
                    0..0,
                    ast::builders::function(
                        0..0,
                        parameter.clone(),
                        ast::builders::infix(
                            0..0,
                            Operation::Add,
                            ast::builders::identifier(0..0, parameter.clone()),
                            ast::builders::primitive_integer(0..0, modifier.clone()),
                        ),
                    ),
                    ast::builders::primitive_integer(0..0, value.clone()),
                );
                let expected = pool_with(|pool| {
                    let left_ref = builders::identifier(pool, parameter.clone());
                    let right_ref = builders::primitive_integer(pool, modifier.clone());
                    let add_ref = builders::infix(pool, Operation::Add, left_ref, right_ref);
                    let function_ref = builders::function(pool, parameter.clone(), add_ref);
                    let argument_ref = builders::primitive_integer(pool, value.clone());
                    builders::apply(pool, function_ref, argument_ref);
                });

                let actual = pool_exprs(input);

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
                let input = ast::builders::infix(
                    0..0,
                    operation,
                    ast::builders::primitive_integer(0..0, left.clone()),
                    ast::builders::primitive_integer(0..0, right.clone()),
                );
                let expected = pool_with(|pool| {
                    let left_ref = builders::primitive_integer(pool, left.clone());
                    let right_ref = builders::primitive_integer(pool, right.clone());
                    builders::infix(pool, operation, left_ref, right_ref);
                });

                let actual = pool_exprs(input);

                prop_assert_eq!(actual, expected);
                Ok(())
            },
        )
    }
}
