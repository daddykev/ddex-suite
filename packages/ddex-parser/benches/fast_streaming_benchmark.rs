// benches/fast_streaming_benchmark.rs
//! Comprehensive benchmarks for FastStreamingParser targeting 280+ MB/s throughput

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ddex_parser::parser::security::SecurityConfig;
use ddex_parser::streaming::{create_fast_parser, FastStreamingParser, StreamingConfig};
use std::io::{BufReader, Cursor};
use std::time::Instant;

/// Generate test XML data of specified size
fn generate_test_xml(num_releases: usize, release_size_kb: usize) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">
    <ern:MessageHeader>
        <ern:MessageThreadId>THREAD_FAST_BENCHMARK</ern:MessageThreadId>
        <ern:MessageId>MSG_PERFORMANCE_TEST</ern:MessageId>
        <ern:MessageFileName>performance_test.xml</ern:MessageFileName>
        <ern:MessageSender>
            <ern:PartyId Namespace="PADPIDA2006">SENDER001</ern:PartyId>
            <ern:PartyName>High Performance Test Sender</ern:PartyName>
        </ern:MessageSender>
        <ern:MessageRecipient>
            <ern:PartyId Namespace="PADPIDA2006">RECIPIENT001</ern:PartyId>
            <ern:PartyName>Fast Streaming Test Recipient</ern:PartyName>
        </ern:MessageRecipient>
        <ern:MessageCreatedDateTime>2024-01-15T10:30:00Z</ern:MessageCreatedDateTime>
        <ern:MessageAuditTrail>Fast streaming performance test</ern:MessageAuditTrail>
    </ern:MessageHeader>
    <ern:ReleaseList>"#,
    );

    // Generate padding content to reach target size
    let padding = "A".repeat(release_size_kb * 1024 / 20); // Distribute across fields

    for i in 0..num_releases {
        xml.push_str(&format!(r#"
        <ern:Release IsMainRelease="true">
            <ern:ReleaseId Namespace="ISRC">{:010}-{:06}</ern:ReleaseId>
            <ern:ReleaseReference>REL{:010}</ern:ReleaseReference>
            <ern:ReferenceTitle>
                <ern:TitleText LanguageAndScriptCode="en">Performance Test Release {} - {}</ern:TitleText>
                <ern:SubTitle LanguageAndScriptCode="en">High Speed Parsing Test Case</ern:SubTitle>
            </ern:ReferenceTitle>
            <ern:ReleaseType>Album</ern:ReleaseType>
            <ern:ReleaseDetailsByTerritory>
                <ern:TerritoryCode>Worldwide</ern:TerritoryCode>
                <ern:DisplayArtist>
                    <ern:PartyName LanguageAndScriptCode="en">Test Artist {}</ern:PartyName>
                    <ern:ArtistRole>MainArtist</ern:ArtistRole>
                </ern:DisplayArtist>
                <ern:LabelName LanguageAndScriptCode="en">Fast Parser Records</ern:LabelName>
                <ern:ReleaseDate>2024-01-15</ern:ReleaseDate>
                <ern:OriginalReleaseDate>2024-01-15</ern:OriginalReleaseDate>
                <ern:PLine>
                    <ern:Year>2024</ern:Year>
                    <ern:PLineCompany>Fast Parser Records Ltd.</ern:PLineCompany>
                    <ern:PLineText>Performance test data generated for benchmarking</ern:PLineText>
                </ern:PLine>
                <ern:CLine>
                    <ern:Year>2024</ern:Year>
                    <ern:CLineCompany>Fast Parser Records Ltd.</ern:CLineCompany>
                    <ern:CLineText>(C) 2024 Fast Parser Records. Benchmark data.</ern:CLineText>
                </ern:CLine>
                <ern:Genre>
                    <ern:GenreText>Electronic/Performance</ern:GenreText>
                    <ern:SubGenre>Benchmark/Synthetic</ern:SubGenre>
                </ern:Genre>
                <ern:ParentalWarningType>NotExplicit</ern:ParentalWarningType>
                <ern:ResourceGroup>
                    <ern:Title>
                        <ern:TitleText LanguageAndScriptCode="en">Performance Test Resource Group</ern:TitleText>
                    </ern:Title>
                    <ern:SequenceNumber>1</ern:SequenceNumber>
                </ern:ResourceGroup>
                <ern:Description LanguageAndScriptCode="en">
                    Generated test data for high-performance streaming parser benchmarks. {padding}
                </ern:Description>
                <ern:Keywords LanguageAndScriptCode="en">performance, benchmark, fast, streaming, parser, test, high-speed</ern:Keywords>
                <ern:Synopsis LanguageAndScriptCode="en">
                    This release contains synthetic test data designed to benchmark the FastStreamingParser performance.
                    Target throughput: 280+ MB/s. Generated with optimized XML structure for memchr optimization.
                </ern:Synopsis>
            </ern:ReleaseDetailsByTerritory>
            <ern:ReleaseResourceReferenceList>
                <ern:ReleaseResourceReference>RES{:010}_01</ern:ReleaseResourceReference>
                <ern:ReleaseResourceReference>RES{:010}_02</ern:ReleaseResourceReference>
                <ern:ReleaseResourceReference>RES{:010}_03</ern:ReleaseResourceReference>
            </ern:ReleaseResourceReferenceList>
        </ern:Release>"#,
        i, i, i, padding, i, i, i, i, i));
    }

    xml.push_str(r#"
    </ern:ReleaseList>
    <ern:ResourceList>
        <ern:SoundRecording>
            <ern:SoundRecordingId Namespace="ISRC">BENCH2024000001</ern:SoundRecordingId>
            <ern:ReferenceTitle>
                <ern:TitleText LanguageAndScriptCode="en">Benchmark Audio Track</ern:TitleText>
            </ern:ReferenceTitle>
            <ern:Duration>PT3M45S</ern:Duration>
            <ern:SoundRecordingDetailsByTerritory>
                <ern:TerritoryCode>Worldwide</ern:TerritoryCode>
                <ern:DisplayArtist>
                    <ern:PartyName LanguageAndScriptCode="en">Performance Test Artist</ern:PartyName>
                    <ern:ArtistRole>MainArtist</ern:ArtistRole>
                </ern:DisplayArtist>
                <ern:LabelName LanguageAndScriptCode="en">Fast Parser Records</ern:LabelName>
                <ern:RightsAgreementId>RA_BENCHMARK_001</ern:RightsAgreementId>
            </ern:SoundRecordingDetailsByTerritory>
        </ern:SoundRecording>
    </ern:ResourceList>
</ern:NewReleaseMessage>"#);

    xml
}

/// Benchmark fast streaming parser with different configurations
fn benchmark_fast_streaming_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("fast_streaming_parser");

    // Test with different data sizes to measure throughput
    let test_cases = vec![
        ("10KB", 10, 1),      // 10 releases, 1KB each
        ("100KB", 100, 1),    // 100 releases, 1KB each
        ("1MB", 250, 4),      // 250 releases, 4KB each
        ("10MB", 500, 20),    // 500 releases, 20KB each
        ("100MB", 1000, 100), // 1000 releases, 100KB each
    ];

    for (size_name, num_releases, kb_per_release) in test_cases {
        let xml = generate_test_xml(num_releases, kb_per_release);
        let data_size = xml.len();

        group.throughput(Throughput::Bytes(data_size as u64));

        group.bench_with_input(
            BenchmarkId::new("throughput", size_name),
            &xml,
            |b, xml_data| {
                b.iter(|| {
                    let mut parser = create_fast_parser();
                    let cursor = Cursor::new(xml_data.as_bytes());
                    let mut reader = BufReader::new(cursor);

                    let result = parser.parse_streaming(&mut reader, None);
                    let iterator = result.expect("Parsing should succeed");

                    // Consume the iterator to measure full performance
                    let elements: Vec<_> = iterator.collect();
                    black_box(elements);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parser configuration variations
fn benchmark_parser_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_configurations");

    let xml = generate_test_xml(500, 10); // 5MB test case
    let data_size = xml.len();
    group.throughput(Throughput::Bytes(data_size as u64));

    // Test different configurations
    let configs = vec![
        (
            "relaxed_security",
            StreamingConfig {
                security: SecurityConfig::relaxed(),
                buffer_size: 64 * 1024,
                max_memory: 200 * 1024 * 1024,
                chunk_size: 512,
                enable_progress: false,
                progress_interval: 0,
            },
        ),
        (
            "strict_security",
            StreamingConfig {
                security: SecurityConfig::strict(),
                buffer_size: 64 * 1024,
                max_memory: 200 * 1024 * 1024,
                chunk_size: 512,
                enable_progress: false,
                progress_interval: 0,
            },
        ),
        (
            "large_buffer",
            StreamingConfig {
                security: SecurityConfig::relaxed(),
                buffer_size: 256 * 1024, // 256KB buffer
                max_memory: 200 * 1024 * 1024,
                chunk_size: 1024, // 1MB chunks
                enable_progress: false,
                progress_interval: 0,
            },
        ),
        (
            "small_buffer",
            StreamingConfig {
                security: SecurityConfig::relaxed(),
                buffer_size: 16 * 1024, // 16KB buffer
                max_memory: 50 * 1024 * 1024,
                chunk_size: 128, // 128KB chunks
                enable_progress: false,
                progress_interval: 0,
            },
        ),
    ];

    for (config_name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("config", config_name),
            &xml,
            |b, xml_data| {
                b.iter(|| {
                    let mut parser = FastStreamingParser::new(config.clone());
                    let cursor = Cursor::new(xml_data.as_bytes());
                    let mut reader = BufReader::new(cursor);

                    let result = parser.parse_streaming(&mut reader, None);
                    let iterator = result.expect("Parsing should succeed");

                    let elements: Vec<_> = iterator.collect();
                    black_box(elements);
                });
            },
        );
    }

    group.finish();
}

/// Validate that target throughput (280+ MB/s) is achieved
fn validate_throughput_target(c: &mut Criterion) {
    let xml = generate_test_xml(1000, 50); // ~50MB test case
    let data_size = xml.len();

    println!("\n=== THROUGHPUT VALIDATION ===");
    println!(
        "Test data size: {:.2} MB",
        data_size as f64 / (1024.0 * 1024.0)
    );

    // Run the test and measure throughput
    let mut parser = create_fast_parser();
    let cursor = Cursor::new(xml.as_bytes());
    let mut reader = BufReader::new(cursor);

    let start = Instant::now();
    let result = parser.parse_streaming(&mut reader, None);
    let parsing_duration = start.elapsed();

    match result {
        Ok(iterator) => {
            let stats = iterator.stats().clone();
            let elements: Vec<_> = iterator.collect();
            let total_duration = start.elapsed();

            let throughput_mbps =
                (data_size as f64) / (1024.0 * 1024.0) / parsing_duration.as_secs_f64();

            println!("Parsing Results:");
            println!("  Elements found: {}", elements.len());
            println!("  Parsing time: {:?}", parsing_duration);
            println!("  Total time: {:?}", total_duration);
            println!("  Throughput: {:.2} MB/s", throughput_mbps);
            println!("  Target: 280+ MB/s");

            if throughput_mbps >= 280.0 {
                println!("  ✅ TARGET ACHIEVED!");
            } else {
                println!(
                    "  ⚠️  Target not reached ({}% of target)",
                    (throughput_mbps / 280.0 * 100.0) as u32
                );
            }

            println!("  Stats from parser: {:#?}", stats);
        }
        Err(e) => {
            println!("  ❌ Parsing failed: {:?}", e);
        }
    }

    // Also run as a criterion benchmark for detailed measurement
    let mut group = c.benchmark_group("throughput_validation");
    group.throughput(Throughput::Bytes(data_size as u64));

    group.bench_function("50MB_target_validation", |b| {
        b.iter(|| {
            let mut parser = create_fast_parser();
            let cursor = Cursor::new(xml.as_bytes());
            let mut reader = BufReader::new(cursor);

            let result = parser.parse_streaming(&mut reader, None);
            let iterator = result.expect("Parsing should succeed");
            let elements: Vec<_> = iterator.collect();
            black_box(elements);
        });
    });

    group.finish();
}

/// Benchmark memory efficiency
fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    let xml = generate_test_xml(2000, 25); // ~50MB, many small elements
    let data_size = xml.len();

    // Report (not assert) the parser's own peak-memory figure. Note: FastStreamingParser
    // does not apply `max_memory` (KNOWN_ISSUES.md).
    {
        let mut parser = FastStreamingParser::new(StreamingConfig {
            security: SecurityConfig::relaxed(),
            buffer_size: 32 * 1024,
            max_memory: 10 * 1024 * 1024,
            chunk_size: 256,
            enable_progress: false,
            progress_interval: 0,
        });
        let mut reader = BufReader::new(Cursor::new(xml.as_bytes()));
        if let Ok(iterator) = parser.parse_streaming(&mut reader, None) {
            println!(
                "memory_efficiency: {:.1} MB input, reported peak memory {:.2} MB",
                data_size as f64 / (1024.0 * 1024.0),
                iterator.stats().peak_memory_mb
            );
        }
    }
    group.throughput(Throughput::Bytes(data_size as u64));

    group.bench_function("memory_bounded_parsing", |b| {
        b.iter(|| {
            let config = StreamingConfig {
                security: SecurityConfig::relaxed(),
                buffer_size: 32 * 1024,       // Smaller buffer
                max_memory: 10 * 1024 * 1024, // 10MB memory limit
                chunk_size: 256,
                enable_progress: false,
                progress_interval: 0,
            };

            let mut parser = FastStreamingParser::new(config);
            let cursor = Cursor::new(xml.as_bytes());
            let mut reader = BufReader::new(cursor);

            let result = parser.parse_streaming(&mut reader, None);
            let iterator = result.expect("Parsing should succeed");
            let stats = iterator.stats().clone();
            let elements: Vec<_> = iterator.collect();
            black_box((elements, stats));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_fast_streaming_parser,
    benchmark_parser_configurations,
    validate_throughput_target,
    benchmark_memory_efficiency
);

criterion_main!(benches);
