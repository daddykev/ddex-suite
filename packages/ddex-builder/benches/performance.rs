use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ddex_builder::{BuildOptions, DDEXBuilder};
use std::time::Duration;

mod common;
use common::request_with_tracks as create_test_request_with_tracks;

// Reports numbers only (recorded in dev-docs/PERFORMANCE.md). tests/it/perf.rs
// (`--features perf-tests`) keeps generous sanity limits against catastrophic regressions.

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
                black_box(result.unwrap())
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
                black_box(result.unwrap())
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
                black_box(result.unwrap())
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
                    black_box(result.unwrap())
                });
            },
        );
    }

    group.finish();
}

/// The `ParallelProcessor` path (formerly the "optimized" perf tests).
fn benchmark_parallel_processor(c: &mut Criterion) {
    use ddex_builder::memory_optimization::BuildMemoryManager;
    use ddex_builder::optimized_strings::BuildContext;
    use ddex_builder::parallel_processing::{ParallelConfig, ParallelProcessor};

    let mut group = c.benchmark_group("parallel_processor");
    for track_count in [1usize, 12, 50] {
        group.bench_with_input(
            BenchmarkId::new("process_build_parallel", format!("{}_tracks", track_count)),
            &track_count,
            |b, &track_count| {
                let request = create_test_request_with_tracks(track_count);
                let processor = ParallelProcessor::new(ParallelConfig::default()).unwrap();
                let memory_manager = BuildMemoryManager::new();
                b.iter(|| {
                    let mut context = BuildContext::new();
                    black_box(
                        processor
                            .process_build_parallel(
                                black_box(&request),
                                &mut context,
                                &memory_manager,
                            )
                            .unwrap(),
                    )
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_performance_scales,
    benchmark_parallel_processor,
    benchmark_memory_allocation,
    benchmark_xml_generation
);
criterion_main!(benches);
