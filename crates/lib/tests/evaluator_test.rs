mod helpers;

use proptest::prelude::*;

use boo::*;
use boo_test_helpers::proptest::*;

#[test]
fn test_evaluation_gets_the_same_result_as_naive_evaluation() {
    check(&parser::generators::arbitrary(), |expr| {
        let expected = crate::helpers::naive_evaluator::naively_evaluate(expr.clone());

        let pooled = pooler::pool_exprs(expr);
        let actual = evaluator::evaluate(&pooled);

        prop_assert_eq!(expected, actual);
        Ok(())
    })
}
