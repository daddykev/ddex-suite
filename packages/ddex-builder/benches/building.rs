use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ddex_builder::{BuildOptions, DDEXBuilder};

mod common;

fn benchmark_building(c: &mut Criterion) {
    c.bench_function("build_simple_release", |b| {
        let builder = DDEXBuilder::new();
        let request = common::request_with_tracks(1);

        b.iter(|| {
            let result = builder.build(
                black_box(request.clone()),
                black_box(BuildOptions::default()),
            );
            black_box(result)
        });
    });
}

criterion_group!(benches, benchmark_building);
criterion_main!(benches);
