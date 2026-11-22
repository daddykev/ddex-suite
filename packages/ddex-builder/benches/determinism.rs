//! Benchmarks for determinism verification overhead
//!
//! These benchmarks measure the cost of `DeterminismVerifier` relative to a
//! single build, so the overhead of verifying output stays visible.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ddex_builder::determinism::{DeterminismConfig, DeterminismVerifier};
use ddex_builder::{BuildOptions, DDEXBuilder};

mod common;

const SIZES: [(&str, usize); 3] = [("small", 5), ("medium", 50), ("large", 200)];

fn bench_single_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_build");
    let builder = DDEXBuilder::new();

    for (name, tracks) in SIZES {
        let request = common::request_with_tracks(tracks);
        group.bench_with_input(BenchmarkId::new("build", name), &request, |b, request| {
            b.iter(|| {
                black_box(
                    builder
                        .build(request.clone(), BuildOptions::default())
                        .unwrap(),
                )
            })
        });
    }

    group.finish();
}

fn bench_determinism_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("determinism_verification");
    let verifier = DeterminismVerifier::new(DeterminismConfig::default());

    for (name, tracks) in SIZES {
        let request = common::request_with_tracks(tracks);
        for iterations in [2usize, 3, 5] {
            group.bench_with_input(
                BenchmarkId::new(format!("verify_{iterations}x"), name),
                &request,
                |b, request| b.iter(|| black_box(verifier.verify(request, iterations).unwrap())),
            );
        }
    }

    group.finish();
}

fn bench_convenience_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("determinism_checks");
    let request = common::request_with_tracks(12);

    group.bench_function("quick_check", |b| {
        b.iter(|| black_box(DeterminismVerifier::quick_check(&request).unwrap()))
    });
    group.bench_function("thorough_check_3x", |b| {
        b.iter(|| black_box(DeterminismVerifier::thorough_check(&request, 3).unwrap()))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_build,
    bench_determinism_verification,
    bench_convenience_checks
);
criterion_main!(benches);
