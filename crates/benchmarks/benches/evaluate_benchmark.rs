use std::rc::Rc;

use criterion::{black_box, BenchmarkId, Criterion};
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;

use boo::parser::generators::ExprGenConfig;
use boo::*;

pub fn evaluate_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate");

    let mut runner = TestRunner::deterministic();
    for i in 0..8 {
        let tree = parser::generators::gen(Rc::new(ExprGenConfig {
            depth: 8..9,
            ..Default::default()
        }))
        .new_tree(&mut runner)
        .unwrap();
        let expr = tree.current();
        let pool = pool_exprs(expr);
        group.bench_with_input(BenchmarkId::new("evaluate", i), &pool, |b, pool| {
            b.iter(|| evaluate(black_box(pool)))
        });
    }
    group.finish();
}
