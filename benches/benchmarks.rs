mod evaluate_benchmark;

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, evaluate_benchmark::evaluate_benchmark);
criterion_main!(benches);
