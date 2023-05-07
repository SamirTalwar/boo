use rand::{RngCore, SeedableRng};
use std::rc::Rc;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use boo::ast::*;
use boo::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut unstructured_bytes = vec![0; 0x10000];
    let mut rng = rand::rngs::StdRng::from_seed([0; 32]);
    rng.fill_bytes(&mut unstructured_bytes);
    let mut unstructured = arbitrary::Unstructured::new(&unstructured_bytes);

    let expr = Rc::new(unstructured.arbitrary::<Expr<()>>().unwrap());
    c.bench_with_input(BenchmarkId::new("evaluate", &expr), &expr, |b, expr| {
        b.iter(|| evaluate(black_box(expr.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
