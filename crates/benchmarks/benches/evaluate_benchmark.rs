use std::iter;

use criterion::{black_box, BenchmarkId, Criterion};
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;

use boo_core::expr::Expr;

const BENCHMARK_COUNT: usize = 8;

pub fn evaluate_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate");
    for (i, expr) in benchmarks().take(BENCHMARK_COUNT).enumerate() {
        group.bench_with_input(BenchmarkId::new("evaluate", i), &expr, |b, expr| {
            b.iter(|| boo_evaluator::evaluate(black_box(expr.clone())))
        });
        group.bench_with_input(BenchmarkId::new("naively evaluate", i), &expr, |b, expr| {
            b.iter(|| boo_naive_evaluator::naively_evaluate(black_box(expr.clone())))
        });
    }
    group.finish();
}

#[allow(dead_code)]
fn main() {
    println!("Benchmarks for `evaluate`:");
    for (i, expr) in benchmarks().take(BENCHMARK_COUNT).enumerate() {
        println!("\nBenchmark #{}:\n{}", i, expr);
    }
}

fn benchmarks() -> impl Iterator<Item = Expr> {
    let mut runner = TestRunner::deterministic();
    iter::from_fn(move || {
        let tree = boo_generator::gen::<Expr>(
            boo_generator::ExprGenConfig {
                depth: 8..9,
                ..Default::default()
            }
            .into(),
        )
        .new_tree(&mut runner)
        .unwrap();
        Some(tree.current())
    })
}
