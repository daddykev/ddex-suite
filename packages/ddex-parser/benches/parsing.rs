// core/benches/parsing.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use ddex_parser_core::{DDEXParser, parser::{ParseOptions, mode::ParseMode}};
use std::io::Cursor;
use std::time::Duration;

fn benchmark_parse_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_by_size");
    group.measurement_time(Duration::from_secs(10));
    
    // Use include_str! for compile-time inclusion
    let xml = include_str!("../../test-suite/valid/ern-4.3/simple_release.xml");
    
    let parser = DDEXParser::new();
    
    let size = xml.len() as u64;
    group.throughput(Throughput::Bytes(size));
    
    group.bench_function("dom", |b| {
        b.iter(|| {
            let options = ParseOptions {
                mode: ParseMode::Dom,
                ..Default::default()
            };
            let cursor = Cursor::new(xml.as_bytes());
            parser.parse_with_options(cursor, options)
        });
    });
    
    group.bench_function("stream", |b| {
        b.iter(|| {
            let options = ParseOptions {
                mode: ParseMode::Stream,
                ..Default::default()
            };
            let cursor = Cursor::new(xml.as_bytes());
            parser.parse_with_options(cursor, options)
        });
    });
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Use include_str! here too
    let xml = include_str!("../../test-suite/valid/ern-4.3/simple_release.xml");
    
    group.bench_function("stream_simple", |b| {
        b.iter(|| {
            use ddex_parser_core::parser::{stream::StreamingParser, detector::ERNVersion};
            
            let mut release_count = 0;
            let reader = Cursor::new(xml.as_bytes());
            let mut streaming_parser = StreamingParser::new(reader, ERNVersion::V4_3);
            
            for release in streaming_parser.stream_releases() {
                let _ = black_box(release); // Handle the Result
                release_count += 1;
            }
            
            release_count
        });
    });
    
    group.finish();
}

fn benchmark_reference_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("reference_resolution");
    
    let parser = DDEXParser::new();
    let xml = include_str!("../../test-suite/valid/ern-4.3/simple_release.xml");
    
    group.bench_function("with_resolution", |b| {
        b.iter(|| {
            let options = ParseOptions {
                resolve_references: true,
                ..Default::default()
            };
            let cursor = Cursor::new(xml.as_bytes());
            parser.parse_with_options(cursor, options)
        });
    });
    
    group.bench_function("without_resolution", |b| {
        b.iter(|| {
            let options = ParseOptions {
                resolve_references: false,
                ..Default::default()
            };
            let cursor = Cursor::new(xml.as_bytes());
            parser.parse_with_options(cursor, options)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_parse_sizes,
    benchmark_memory_usage,
    benchmark_reference_resolution
);
criterion_main!(benches);