// core/benches/memory.rs
use criterion::{criterion_group, criterion_main, Criterion};
use std::io::Cursor;

fn benchmark_memory(c: &mut Criterion) {
    // Go up TWO levels
    let xml = include_str!("../tests/fixtures/ern43_basic.xml");

    c.bench_function("memory_bounded_parse", |b| {
        b.iter(|| {
            use ddex_parser::{
                parser::{mode::ParseMode, ParseOptions},
                DDEXParser,
            };
            let mut parser = DDEXParser::new();
            let options = ParseOptions {
                mode: ParseMode::Stream,
                max_memory: 10 * 1024 * 1024, // 10MB limit
                ..Default::default()
            };
            let cursor = Cursor::new(xml.as_bytes());
            parser.parse_with_options(cursor, options)
        });
    });
}

criterion_group!(benches, benchmark_memory);
criterion_main!(benches);
