use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;

use criterion::{black_box, BenchmarkId, Criterion};

use boo::*;

pub fn evaluate_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate");

    let mut runner = TestRunner::deterministic();
    for _ in 0..16 {
        let tree = parser::generators::arbitrary()
            .new_tree(&mut runner)
            .unwrap();
        let expr = tree.current();
        group.bench_with_input(BenchmarkId::new("evaluate", &expr), &expr, |b, expr| {
            b.iter(|| evaluate(black_box(expr.clone())))
        });
    }
    group.finish();
}
