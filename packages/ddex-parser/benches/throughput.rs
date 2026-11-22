//! Parser throughput: the official ERN 4.3 samples, a ~10 MB catalog, and the streaming
//! iterators (working, zero-copy, parallel by thread count). Reports numbers only;
//! `tests/it/perf.rs` (`--features perf-tests`) keeps a few generous sanity limits.
//!
//!     cargo bench -p ddex-parser --bench throughput

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ddex_core::models::versions::ERNVersion;
use ddex_parser::streaming::{
    FastZeroCopyIterator, ParallelStreamingIterator, WorkingStreamIterator,
};
use ddex_parser::DDEXParser;
use std::io::Cursor;
use std::time::Duration;

mod common;

const CATALOG_BYTES: usize = 10 * 1024 * 1024;

fn official_samples(c: &mut Criterion) {
    let mut group = c.benchmark_group("official_samples_43");
    for (name, xml) in common::official_43_samples() {
        group.throughput(Throughput::Bytes(xml.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&name), &xml, |b, xml| {
            let mut parser = DDEXParser::new();
            b.iter(|| {
                parser
                    .parse(Cursor::new(black_box(xml.as_bytes())))
                    .unwrap()
            });
        });
    }
    group.finish();
}

fn large_catalog(c: &mut Criterion) {
    let xml = common::large_catalog_43(CATALOG_BYTES);
    let mut group = c.benchmark_group("large_catalog_43");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));
    group.throughput(Throughput::Bytes(xml.len() as u64));
    group.bench_function("parse_10MB", |b| {
        let mut parser = DDEXParser::new();
        b.iter(|| {
            parser
                .parse(Cursor::new(black_box(xml.as_bytes())))
                .unwrap()
        });
    });
    group.finish();
}

fn streaming_iterators(c: &mut Criterion) {
    let xml = common::large_catalog_43(CATALOG_BYTES);
    let bytes = xml.as_bytes();
    let mut group = c.benchmark_group("streaming_iterators_10MB");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));
    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("working", |b| {
        b.iter(|| WorkingStreamIterator::new(Cursor::new(bytes), ERNVersion::V4_3).count())
    });
    group.bench_function("zero_copy", |b| {
        b.iter(|| FastZeroCopyIterator::new(Cursor::new(bytes), ERNVersion::V4_3).count())
    });
    for threads in [1, 2, 4, 8] {
        if threads > num_cpus::get() {
            continue;
        }
        group.bench_with_input(
            BenchmarkId::new("parallel", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    ParallelStreamingIterator::with_threads(
                        Cursor::new(bytes),
                        ERNVersion::V4_3,
                        threads,
                    )
                    .count()
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    official_samples,
    large_catalog,
    streaming_iterators
);
criterion_main!(benches);
