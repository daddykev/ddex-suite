//! Cost of path validation (formerly the `security_fixes` overhead test), next to a trivial
//! baseline. Reports numbers only. (The entity-classification and error-sanitization groups
//! were removed with those helpers in 0.6.0.)
//!
//!     cargo bench -p ddex-builder --bench security

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ddex_builder::security::PathValidator;

const PATHS: [&str; 4] = [
    "valid/path.xml",
    "another/valid/path.json",
    "../invalid/path.txt",
    "normal/file.mp3",
];

fn path_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_validation_4_paths");
    group.bench_function("baseline_path_new", |b| {
        b.iter(|| {
            for p in PATHS {
                black_box(std::path::Path::new(black_box(p)).components().count());
            }
        })
    });
    let validator = PathValidator::new();
    group.bench_function("path_validator", |b| {
        b.iter(|| {
            for p in PATHS {
                let _ = black_box(validator.validate(black_box(p)));
            }
        })
    });
    group.finish();
}

criterion_group!(benches, path_validation);
criterion_main!(benches);
