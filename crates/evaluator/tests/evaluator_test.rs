use proptest::prelude::*;

use boo_core::ast::ExpressionWrapper;
use boo_test_helpers::proptest::*;

#[test]
fn test_evaluation_gets_the_same_result_as_naive_evaluation() {
    check(&boo_generator::arbitrary(), |expr| {
        let parsed =
            expr.transform(&mut |_, expression| boo_parser::Expr::new(0.into(), expression));
        let expected = boo_naive_evaluator::naively_evaluate(parsed.clone()).unwrap();
        let actual = boo_evaluator::evaluate(parsed).unwrap();

        match (expected, actual) {
            (
                boo_naive_evaluator::Evaluated::Primitive(expected),
                boo_evaluator::Evaluated::Primitive(actual),
            ) => {
                prop_assert_eq!(expected, actual);
            }
            (expected, actual) => panic!(
                "Test failed: assertion failed: `(left == right)`\n  left:   `{}`,\n  right:  `{}`",
                expected, actual
            ),
        }
        Ok(())
    })
}
