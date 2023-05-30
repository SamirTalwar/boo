use num_bigint::BigInt;
use proptest::prelude::*;
use proptest::strategy::{Strategy, ValueTree};
use proptest::test_runner::TestRunner;

use criterion::{black_box, BenchmarkId, Criterion};

use boo_core::primitive::Integer;

pub fn integer_benchmark(c: &mut Criterion) {
    let mut runner = TestRunner::deterministic();

    {
        let mut group = c.benchmark_group("integer/small");
        for _ in 0..4 {
            let x = any::<i16>().new_tree(&mut runner).unwrap().current();
            let y = any::<i16>().new_tree(&mut runner).unwrap().current();
            group.bench_with_input(
                BenchmarkId::new("Integer", format!("{} + {}", x, y)),
                &(Integer::from(x), Integer::from(y)),
                |b, (x, y)| b.iter(|| black_box(x) + black_box(y)),
            );
            group.bench_with_input(
                BenchmarkId::new("i32", format!("{} + {}", x, y)),
                &(i32::from(x), i32::from(y)),
                |b, (x, y)| b.iter(|| black_box(x) + black_box(y)),
            );
            group.bench_with_input(
                BenchmarkId::new("BigInt", format!("{} + {}", x, y)),
                &(BigInt::from(x), BigInt::from(y)),
                |b, (x, y)| b.iter(|| black_box(x) + black_box(y)),
            );
        }
        group.finish();
    }

    {
        let mut group = c.benchmark_group("integer/large");
        for _ in 0..4 {
            let x = any::<i128>().new_tree(&mut runner).unwrap().current();
            let y = any::<i128>().new_tree(&mut runner).unwrap().current();
            group.bench_with_input(
                BenchmarkId::new("Integer", format!("{} * {}", x, y)),
                &(Integer::from(x), Integer::from(y)),
                |b, (x, y)| b.iter(|| black_box(x) * black_box(y)),
            );
            group.bench_with_input(
                BenchmarkId::new("BigInt", format!("{} * {}", x, y)),
                &(BigInt::from(x), BigInt::from(y)),
                |b, (x, y)| b.iter(|| black_box(x) * black_box(y)),
            );
        }
        group.finish();
    }
}
