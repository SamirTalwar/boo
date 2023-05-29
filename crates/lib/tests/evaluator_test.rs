mod helpers;

use proptest::prelude::*;

use boo::*;
use boo_test_helpers::proptest::*;

use crate::helpers::naive_evaluator;

#[test]
#[ignore = "functions are not implemented yet"]
fn test_evaluation_gets_the_same_result_as_naive_evaluation() {
    check(&parser::generators::arbitrary(), |expr| {
        let expected = naive_evaluator::naively_evaluate(expr.clone()).unwrap();

        let pooled = pooler::pool_exprs(expr);
        let actual = evaluator::evaluate(&pooled).unwrap();

        match (expected, actual) {
            (
                parser::ast::Expression::Primitive(expected),
                pooler::ast::Expression::Primitive(actual),
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
