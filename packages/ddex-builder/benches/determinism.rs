//! Single-build benchmarks at 5, 50 and 200 tracks (group `single_build`). Its
//! `DeterminismVerifier` groups were removed with the verifier in 0.6.0.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
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

criterion_group!(benches, bench_single_build);
criterion_main!(benches);
