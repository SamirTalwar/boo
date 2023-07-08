use proptest::prelude::*;

use boo_core::ast::*;
use boo_core::builtins;
use boo_test_helpers::proptest::*;

#[test]
fn test_evaluation_gets_the_same_result_as_naive_evaluation() {
    check(&boo_generator::arbitrary::<boo_parser::Expr>(), |expr| {
        let prepared = builtins::prepare(expr.clone());
        let expected = boo_naive_evaluator::naively_evaluate(prepared.clone());
        let actual = boo_evaluator::evaluate(prepared);

        match (expected.map(|e| e.expression()), actual) {
            (
                Ok(Expression::Primitive(expected)),
                Ok(boo_evaluator::Evaluated::Primitive(actual)),
            ) => {
                prop_assert_eq!(expected, actual);
            }
            (Ok(expected), Ok(actual)) => prop_assert!(
                false,
                "did not finish evaluation\n  left:   `{}`,\n  right:  `{}`\n  input:  {}\n",
                expected,
                actual,
                expr
            ),
            (expected, actual) => prop_assert!(
                false,
                "evaulation failed\n  left:   `{:?}`,\n  right:  `{:?}`\n  input:  {}\n",
                expected,
                actual,
                expr
            ),
        }
        Ok(())
    })
}
