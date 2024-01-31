use proptest::prelude::*;

use boo_core::builtins;
use boo_core::evaluation::*;
use boo_test_helpers::proptest::*;

#[test]
fn test_evaluation_gets_the_same_result_as_reducing_evaluation() {
    let mut reducing_evaluator = boo_evaluation_reduction::new();
    builtins::prepare(&mut reducing_evaluator).unwrap();
    let mut optimized_evaluator = boo_evaluation_optimized::new();
    builtins::prepare(&mut optimized_evaluator).unwrap();

    check(&boo_generator::arbitrary(), |expr| {
        let core_expr = expr.clone().to_core()?;
        let expected = reducing_evaluator.evaluate(core_expr.clone());
        let actual = optimized_evaluator.evaluate(core_expr);

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
