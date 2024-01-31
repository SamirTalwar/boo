use std::iter;

use criterion::{black_box, BenchmarkId, Criterion};
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;

use boo_core::builtins;
use boo_core::evaluation::Evaluator;
use boo_core::expr::Expr;

const BENCHMARK_COUNT: usize = 8;

pub fn evaluate_benchmark(c: &mut Criterion) {
    let evaluators: Vec<(String, Box<dyn Evaluator>)> = vec![
        (
            "reduction".to_owned(),
            prepare(boo_evaluation_reduction::new()),
        ),
        (
            "recursive".to_owned(),
            prepare(boo_evaluation_recursive::new()),
        ),
        (
            "optimized".to_owned(),
            prepare(boo_evaluation_optimized::new()),
        ),
    ];

    let mut group = c.benchmark_group("evaluate");
    for (i, expr) in benchmarks().take(BENCHMARK_COUNT).enumerate() {
        for (name, evaluator) in evaluators.iter() {
            group.bench_with_input(BenchmarkId::new(name, i), &expr, |b, expr| {
                b.iter(|| evaluator.evaluate(black_box(expr.clone())).unwrap())
            });
        }
    }
    group.finish();
}

fn prepare(mut evaluator: impl Evaluator + 'static) -> Box<dyn Evaluator> {
    builtins::prepare(&mut evaluator).unwrap();
    Box::new(evaluator)
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
        let tree = boo_generator::gen(
            boo_generator::ExprGenConfig {
                depth: 8..9,
                ..Default::default()
            }
            .into(),
        )
        .new_tree(&mut runner)
        .unwrap();
        Some(tree.current().to_core().unwrap())
    })
}
