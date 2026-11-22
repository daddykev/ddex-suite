use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ddex_builder::{BuildOptions, DDEXBuilder};
use std::time::Duration;

mod common;
use common::request_with_tracks as create_test_request_with_tracks;

// Target metrics:
// - Single track: <5ms
// - 12-track album: <10ms
// - 100-track compilation: <50ms

fn benchmark_performance_scales(c: &mut Criterion) {
    let mut group = c.benchmark_group("ddex_builder_performance");

    // Set reasonable measurement time
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Small: Single track (~5KB output)
    group.bench_with_input(
        BenchmarkId::new("single_track", "1_track"),
        &1,
        |b, &track_count| {
            let builder = DDEXBuilder::new();
            let request = create_test_request_with_tracks(track_count);

            b.iter(|| {
                let result = builder.build(
                    black_box(request.clone()),
                    black_box(BuildOptions::default()),
                );
                black_box(result)
            });
        },
    );

    // Medium: 12-track album (~60KB output) - TARGET: <10ms
    group.bench_with_input(
        BenchmarkId::new("typical_album", "12_tracks"),
        &12,
        |b, &track_count| {
            let builder = DDEXBuilder::new();
            let request = create_test_request_with_tracks(track_count);

            b.iter(|| {
                let result = builder.build(
                    black_box(request.clone()),
                    black_box(BuildOptions::default()),
                );
                black_box(result)
            });
        },
    );

    // Large: 100-track compilation (~500KB output) - TARGET: <50ms
    group.bench_with_input(
        BenchmarkId::new("large_compilation", "100_tracks"),
        &100,
        |b, &track_count| {
            let builder = DDEXBuilder::new();
            let request = create_test_request_with_tracks(track_count);

            b.iter(|| {
                let result = builder.build(
                    black_box(request.clone()),
                    black_box(BuildOptions::default()),
                );
                black_box(result)
            });
        },
    );

    group.finish();
}

fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    // Benchmark builder creation overhead
    group.bench_function("builder_creation", |b| {
        b.iter(|| black_box(DDEXBuilder::new()));
    });

    // Benchmark request cloning (measures serialization overhead)
    group.bench_function("request_clone_12_tracks", |b| {
        let request = create_test_request_with_tracks(12);
        b.iter(|| black_box(request.clone()));
    });

    group.finish();
}

fn benchmark_xml_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("xml_generation");

    // Test different components of XML generation
    for track_count in [1, 12, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("xml_write", format!("{}_tracks", track_count)),
            track_count,
            |b, &track_count| {
                let builder = DDEXBuilder::new();
                let request = create_test_request_with_tracks(track_count);

                b.iter(|| {
                    // Only measure the XML serialization part
                    let result = builder.build(
                        black_box(request.clone()),
                        black_box(BuildOptions::default()),
                    );
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_performance_scales,
    benchmark_memory_allocation,
    benchmark_xml_generation
);
criterion_main!(benches);
