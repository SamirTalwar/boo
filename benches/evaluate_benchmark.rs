use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;
use std::rc::Rc;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use boo::ast::*;
use boo::*;

fn evaluate_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate");

    let mut runner = TestRunner::deterministic();
    for _ in 0..16 {
        let tree = Expr::arbitrary().new_tree(&mut runner).unwrap();
        let expr = Rc::new(tree.current());
        group.bench_with_input(BenchmarkId::new("evaluate", &expr), &expr, |b, expr| {
            b.iter(|| evaluate(black_box(expr.clone())))
        });
    }
    group.finish();
}

criterion_group!(benches, evaluate_benchmark);
criterion_main!(benches);
