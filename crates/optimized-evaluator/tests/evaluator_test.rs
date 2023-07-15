use proptest::prelude::*;

use boo_core::builtins;
use boo_core::evaluation::*;
use boo_naive_evaluator::NaiveEvaluator;
use boo_optimized_evaluator::OptimizedEvaluator;
use boo_test_helpers::proptest::*;

#[test]
fn test_evaluation_gets_the_same_result_as_naive_evaluation() {
    let naive_evaluator = NaiveEvaluator::new();
    let optimized_evaluator = OptimizedEvaluator::new();

    check(&boo_generator::arbitrary(), |expr| {
        let rewritten = boo_parser::rewrite(expr.clone());
        let prepared = builtins::prepare(rewritten);
        let expected = naive_evaluator.evaluate(prepared.clone());
        let actual = optimized_evaluator.evaluate(prepared);

        match (expected, actual) {
            (Ok(Evaluated::Primitive(expected)), Ok(Evaluated::Primitive(actual))) => {
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
