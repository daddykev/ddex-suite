//! Benchmarks for determinism verification overhead
//!
//! These benchmarks measure the performance impact of determinism verification
//! to ensure the overhead remains acceptable for production use.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ddex_builder::determinism::{DeterminismConfig, DeterminismVerifier};
use ddex_builder::{Builder, BuildRequest};
use serde_json::json;

fn create_benchmark_request(size: &str) -> BuildRequest {
    let track_count = match size {
        "small" => 5,
        "medium" => 50,
        "large" => 200,
        "xlarge" => 1000,
        _ => 10,
    };
    
    BuildRequest {
        data: json!({
            "messageType": "NewReleaseMessage",
            "messageId": format!("MSG_{}", size),
            "release": {
                "releaseId": format!("REL_{}", size),
                "title": format!("Benchmark Album - {}", size),
                "artist": "Benchmark Artist",
                "tracks": (0..track_count).map(|i| {
                    json!({
                        "trackId": format!("TRK{:03}", i),
                        "title": format!("Track {}", i + 1),
                        "duration": format!("PT{}M{}S", (i % 5) + 2, (i * 7) % 60),
                        "isrc": format!("USRC17{:06}", i),
                        "contributors": [
                            {
                                "name": format!("Artist {}", i % 3),
                                "role": "MainArtist"
                            }
                        ]
                    })
                }).collect::<Vec<_>>()
            }
        }),
        config: DeterminismConfig::default(),
        preset: None,
        validate: false,
    }
}

fn bench_single_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_build");
    
    for size in &["small", "medium", "large"] {
        let request = create_benchmark_request(size);
        let builder = Builder::new();
        
        group.bench_with_input(BenchmarkId::new("build", size), size, |b, _| {
            b.iter(|| {
                let result = builder.build(&request);
                black_box(result)
            })
        });
    }
    
    group.finish();
}

fn bench_determinism_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("determinism_verification");
    
    for size in &["small", "medium", "large"] {
        let request = create_benchmark_request(size);
        let config = DeterminismConfig::default();
        let verifier = DeterminismVerifier::new(config);
        
        for iterations in &[2, 3, 5, 10] {
            group.bench_with_input(
                BenchmarkId::new(format!("{}_iters", iterations), size), 
                &(size, iterations), 
                |b, (size, iterations)| {
                    b.iter(|| {
                        let result = verifier.verify(&request, **iterations);
                        black_box(result)
                    })
                }
            );
        }
    }
    
    group.finish();
}

fn bench_determinism_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("determinism_overhead");
    
    let request = create_benchmark_request("medium");
    let builder = Builder::new();
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    // Benchmark single build
    group.bench_function("single_build", |b| {
        b.iter(|| {
            let result = builder.build(&request);
            black_box(result)
        })
    });
    
    // Benchmark 3-iteration verification (most common case)
    group.bench_function("verify_3_iterations", |b| {
        b.iter(|| {
            let result = verifier.verify(&request, 3);
            black_box(result)
        })
    });
    
    // Benchmark quick check
    group.bench_function("quick_check", |b| {
        b.iter(|| {
            let result = DeterminismVerifier::quick_check(&request);
            black_box(result)
        })
    });
    
    group.finish();
}

fn bench_hash_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_calculation");
    
    let sizes = [
        ("1kb", "a".repeat(1024)),
        ("10kb", "a".repeat(10 * 1024)),
        ("100kb", "a".repeat(100 * 1024)),
        ("1mb", "a".repeat(1024 * 1024)),
    ];
    
    for (size_name, data) in &sizes {
        group.bench_with_input(BenchmarkId::new("sha256", size_name), data, |b, data| {
            b.iter(|| {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(data.as_bytes());
                let hash = hasher.finalize();
                black_box(format!("{:x}", hash))
            })
        });
        
        group.bench_with_input(BenchmarkId::new("blake3", size_name), data, |b, data| {
            b.iter(|| {
                let hash = blake3::hash(data.as_bytes());
                black_box(hash.to_hex().to_string())
            })
        });
    }
    
    group.finish();
}

fn bench_stress_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_tests");
    group.sample_size(10); // Reduce sample size for stress tests
    
    let request = create_benchmark_request("medium");
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    group.bench_function("hashmap_stress", |b| {
        b.iter(|| {
            let result = verifier.verify_with_hashmap_stress(&request, 5);
            black_box(result)
        })
    });
    
    group.bench_function("thorough_check", |b| {
        b.iter(|| {
            let result = DeterminismVerifier::thorough_check(&request, 5);
            black_box(result)
        })
    });
    
    group.finish();
}

fn bench_difference_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("difference_analysis");
    
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config).with_context_chars(200);
    
    // Create two similar but different XML strings for difference analysis
    let xml1 = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage>
    <MessageHeader>
        <MessageId>MSG001</MessageId>
        <SentDateTime>2024-01-01T00:00:00Z</SentDateTime>
    </MessageHeader>
    <Release>
        <ReleaseId>REL001</ReleaseId>
        <Title>Test Album</Title>
        <Artist>Test Artist</Artist>
    </Release>
</NewReleaseMessage>"#;
    
    let xml2 = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage>
    <MessageHeader>
        <MessageId>MSG001</MessageId>
        <SentDateTime>2024-01-01T00:00:01Z</SentDateTime>
    </MessageHeader>
    <Release>
        <ReleaseId>REL001</ReleaseId>
        <Title>Test Album</Title>
        <Artist>Test Artist Different</Artist>
    </Release>
</NewReleaseMessage>"#;
    
    group.bench_function("find_first_difference", |b| {
        b.iter(|| {
            let pos = xml1.bytes().zip(xml2.bytes()).position(|(x, y)| x != y);
            black_box(pos)
        })
    });
    
    group.bench_function("calculate_line_col", |b| {
        b.iter(|| {
            let pos = 150; // Approximate position of difference
            let before_pos = &xml1[..pos];
            let line_num = before_pos.lines().count();
            let last_line_start = before_pos.rfind('\n').map(|i| i + 1).unwrap_or(0);
            let col_num = pos - last_line_start + 1;
            black_box((line_num, col_num))
        })
    });
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    let config = DeterminismConfig::default();
    
    // Test with and without output retention
    let verifier_no_outputs = DeterminismVerifier::new(config.clone());
    let verifier_with_outputs = DeterminismVerifier::new(config).with_outputs_retained();
    
    let request = create_benchmark_request("large");
    
    group.bench_function("without_output_retention", |b| {
        b.iter(|| {
            let result = verifier_no_outputs.verify(&request, 5);
            black_box(result)
        })
    });
    
    group.bench_function("with_output_retention", |b| {
        b.iter(|| {
            let result = verifier_with_outputs.verify(&request, 5);
            black_box(result)
        })
    });
    
    group.finish();
}

fn bench_scaling_characteristics(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling_characteristics");
    group.sample_size(20);
    
    let request = create_benchmark_request("medium");
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    // Test how verification time scales with number of iterations
    for iterations in &[2, 3, 5, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::new("iterations", iterations), 
            iterations, 
            |b, iterations| {
                b.iter(|| {
                    let result = verifier.verify(&request, *iterations);
                    black_box(result)
                })
            }
        );
    }
    
    // Test how verification time scales with data size
    for size in &["small", "medium", "large", "xlarge"] {
        let request = create_benchmark_request(size);
        group.bench_with_input(
            BenchmarkId::new("data_size", size), 
            size, 
            |b, _| {
                b.iter(|| {
                    let result = verifier.verify(&request, 3);
                    black_box(result)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_real_world_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_scenarios");
    
    // Simulate a typical CI/CD pipeline verification
    group.bench_function("ci_cd_verification", |b| {
        let request = create_benchmark_request("medium");
        let config = DeterminismConfig::default();
        let verifier = DeterminismVerifier::new(config);
        
        b.iter(|| {
            // Quick check first (most common case)
            let is_deterministic = DeterminismVerifier::quick_check(&request);
            
            // If that passes, do a more thorough check
            if is_deterministic.unwrap_or(false) {
                let result = verifier.verify(&request, 5);
                black_box(result)
            } else {
                black_box(is_deterministic)
            }
        })
    });
    
    // Simulate development workflow verification
    group.bench_function("development_verification", |b| {
        let request = create_benchmark_request("small");
        
        b.iter(|| {
            let result = DeterminismVerifier::quick_check(&request);
            black_box(result)
        })
    });
    
    // Simulate production pre-deployment verification
    group.bench_function("production_verification", |b| {
        let request = create_benchmark_request("large");
        
        b.iter(|| {
            let result = DeterminismVerifier::thorough_check(&request, 10);
            black_box(result)
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_build,
    bench_determinism_verification,
    bench_determinism_overhead,
    bench_hash_calculation,
    bench_stress_tests,
    bench_difference_analysis,
    bench_memory_usage,
    bench_scaling_characteristics,
    bench_real_world_scenarios
);

criterion_main!(benches);