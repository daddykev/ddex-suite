use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ddex_builder::{DB_C14N, DeterminismConfig};

fn benchmark_canonicalization(c: &mut Criterion) {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>TEST_001</MessageId>
    </MessageHeader>
</ern:NewReleaseMessage>"#;

    c.bench_function("canonicalize_small", |b| {
        let canonicalizer = DB_C14N::new(DeterminismConfig::default());
        
        b.iter(|| {
            let result = canonicalizer.canonicalize(black_box(xml));
            black_box(result)
        });
    });
    
    c.bench_function("canonical_hash", |b| {
        let canonicalizer = DB_C14N::new(DeterminismConfig::default());
        
        b.iter(|| {
            let result = canonicalizer.canonical_hash(black_box(xml));
            black_box(result)
        });
    });
}

criterion_group!(benches, benchmark_canonicalization);
criterion_main!(benches);