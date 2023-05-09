use rand::{RngCore, SeedableRng};
use std::rc::Rc;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use boo::ast::*;
use boo::*;

fn evaluate_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluate");
    for seed_value in 0..16 {
        let mut seed = [0; 32];
        seed[0] = seed_value;
        let mut rng = rand::rngs::StdRng::from_seed(seed);
        let mut unstructured_bytes = vec![0; 0x10000];
        rng.fill_bytes(&mut unstructured_bytes);
        let mut unstructured = arbitrary::Unstructured::new(&unstructured_bytes);

        let expr = Rc::new(unstructured.arbitrary::<Expr<()>>().unwrap());
        group.bench_with_input(BenchmarkId::new("evaluate", &expr), &expr, |b, expr| {
            b.iter(|| evaluate(black_box(expr.clone())))
        });
    }
    group.finish();
}

criterion_group!(benches, evaluate_benchmark);
criterion_main!(benches);
