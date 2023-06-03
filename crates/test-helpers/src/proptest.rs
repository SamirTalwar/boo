use proptest::strategy::Strategy;
use proptest::test_runner::{TestCaseResult, TestRunner};

/// Runs a property-based test for the given strategy.
///
/// Similar to the `proptest!` macro, but this is not a macro.
pub fn check<S: Strategy>(strategy: &S, test: impl Fn(S::Value) -> TestCaseResult) {
    let mut runner = TestRunner::default();
    match runner.run(strategy, test) {
        Ok(()) => (),
        Err(e) => panic!("{}\n{}", e, runner),
    }
}
