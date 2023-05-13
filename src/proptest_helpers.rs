#![cfg(test)]

use proptest::strategy::Strategy;
use proptest::test_runner::{TestCaseResult, TestRunner};

pub fn check<S: Strategy>(strategy: &S, test: impl Fn(S::Value) -> TestCaseResult) {
    let mut runner = TestRunner::default();
    match runner.run(strategy, test) {
        Ok(()) => (),
        Err(e) => panic!("{}\n{}", e, runner),
    }
}
