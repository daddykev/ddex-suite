//! Cost of the security helpers (formerly the `security_fixes` overhead test): path
//! validation, entity classification and error sanitization, each next to a trivial
//! baseline. Reports numbers only.
//!
//!     cargo bench -p ddex-builder --bench security

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ddex_builder::security::{
    create_entity, EntityClassifier, ErrorContext, ErrorSanitizer, PathValidator,
};
use std::io::{Error, ErrorKind};

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

fn entity_classification(c: &mut Criterion) {
    let entities = vec![
        create_entity("title", "Song Title"),
        create_entity("artist", "Artist Name"),
        create_entity("isrc", "USRC17607839"),
    ];
    let mut group = c.benchmark_group("entity_classification_3_entities");
    group.bench_function("baseline_lengths", |b| {
        b.iter(|| {
            for e in &entities {
                black_box(e.name.len() + e.value.len());
            }
        })
    });
    let mut classifier = EntityClassifier::new();
    group.bench_function("validate_entity_chain", |b| {
        b.iter(|| black_box(classifier.validate_entity_chain(black_box(&entities))))
    });
    group.finish();
}

fn error_sanitization(c: &mut Criterion) {
    let errors = [
        (ErrorKind::NotFound, "File not found"),
        (ErrorKind::PermissionDenied, "Permission denied"),
        (ErrorKind::InvalidData, "Invalid data"),
    ];
    let mut group = c.benchmark_group("error_sanitization_3_errors");
    group.bench_function("baseline_format", |b| {
        b.iter(|| {
            for (kind, msg) in errors {
                black_box(format!("Error: {}", Error::new(kind, msg)));
            }
        })
    });
    let mut sanitizer = ErrorSanitizer::new();
    group.bench_function("sanitize_io_error", |b| {
        b.iter(|| {
            for (kind, msg) in errors {
                black_box(
                    sanitizer.sanitize_io_error(Error::new(kind, msg), ErrorContext::FileRead),
                );
            }
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    path_validation,
    entity_classification,
    error_sanitization
);
criterion_main!(benches);
