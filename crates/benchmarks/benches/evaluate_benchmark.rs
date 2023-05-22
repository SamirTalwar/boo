use std::iter;
use std::rc::Rc;

use criterion::{black_box, BenchmarkId, Criterion};
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;

use boo::parser::ast::Expr;
use boo::parser::generators::ExprGenConfig;
use boo::*;

const BENCHMARK_COUNT: usize = 8;

pub fn evaluate_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate");
    for (i, expr) in benchmarks().take(BENCHMARK_COUNT).enumerate() {
        let pool = pool_exprs(expr);
        group.bench_with_input(BenchmarkId::new("evaluate", i), &pool, |b, pool| {
            b.iter(|| evaluate(black_box(pool)))
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
        let tree = parser::generators::gen(Rc::new(ExprGenConfig {
            depth: 8..9,
            ..Default::default()
        }))
        .new_tree(&mut runner)
        .unwrap();
        Some(tree.current())
    })
}
