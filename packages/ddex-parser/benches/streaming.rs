// core/benches/streaming.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ddex_parser_core::parser::{stream::StreamingParser, detector::ERNVersion};
use std::io::Cursor;

fn benchmark_streaming(c: &mut Criterion) {
    // Go up TWO levels from core/benches/ to reach test-suite/
    let xml = include_str!("../../test-suite/valid/ern-4.3/simple_release.xml");
    
    c.bench_function("stream_releases", |b| {
        b.iter(|| {
            let cursor = Cursor::new(xml.as_bytes());
            let mut parser = StreamingParser::new(cursor, ERNVersion::V4_3);
            let mut count = 0;
            for release in parser.stream_releases() {
                let _ = black_box(release);
                count += 1;
            }
            count
        });
    });
}

criterion_group!(benches, benchmark_streaming);
criterion_main!(benches);