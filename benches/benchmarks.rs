mod evaluate_benchmark;
mod primitive;

use criterion::{criterion_group, criterion_main};

criterion_group!(
    benches,
    evaluate_benchmark::evaluate_benchmark,
    primitive::integer_benchmark::integer_benchmark
);
criterion_main!(benches);
