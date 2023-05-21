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
        let expr_string = format!("{}", expr);
        let pool = pool_exprs(expr);
        group.bench_with_input(
            BenchmarkId::new("evaluate", &expr_string),
            &pool,
            |b, pool| b.iter(|| evaluate(black_box(pool))),
        );
    }
    group.finish();
}
