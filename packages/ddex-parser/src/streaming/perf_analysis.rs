//! Performance analysis and profiling for streaming parser

use crate::error::ParseError;
use crate::streaming::working_impl::WorkingStreamIterator;
use ddex_core::models::versions::ERNVersion;
use std::io::Cursor;
use std::time::Instant;

/// Generate large test XML for performance testing
fn generate_large_test_file(target_size_bytes: usize) -> Vec<u8> {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>PERF-TEST-MSG</MessageId>
        <CreatedDateTime>2023-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>
"#,
    );

    // Calculate how many releases we need
    let single_release_size = 500; // Approximate size of one release
    let target_releases = (target_size_bytes / single_release_size).max(10);

    println!(
        "Generating {} releases for performance test",
        target_releases
    );

    for i in 0..target_releases {
        xml.push_str(&format!(
            r#"
    <Release ReleaseReference="PERF-REL-{:06}">
        <ReferenceTitle>
            <TitleText>Performance Test Release #{}</TitleText>
            <SubTitle>Benchmark Dataset</SubTitle>
        </ReferenceTitle>
        <Genre>
            <GenreText>Electronic</GenreText>
            <SubGenre>Ambient</SubGenre>
        </Genre>
        <PLine>
            <Year>2023</Year>
            <PLineText>℗ 2023 Test Label</PLineText>
        </PLine>
        <CLine>
            <Year>2023</Year>
            <CLineText>© 2023 Test Label</CLineText>
        </CLine>
        <ReleaseLabelReference>LBL-{:03}</ReleaseLabelReference>
    </Release>
"#,
            i,
            i,
            i % 100
        ));

        // Add some sound recordings too
        for j in 0..5 {
            let resource_ref = format!("PERF-RES-{:06}-{:02}", i, j);
            let isrc = format!("PERF{:08}", i * 100 + j);
            let track_title = format!("Track {} from Release {}", j + 1, i);
            let duration_min = (i * j + 180) % 10;
            let duration_sec = i % 60;
            let party_id = format!("00{:08}", i % 100);
            let artist_name = format!("Perf Artist {}", i % 1000);

            xml.push_str(&format!(
                r#"
    <SoundRecording ResourceReference="{}">
        <ResourceId>
            <ISRC>{}</ISRC>
        </ResourceId>
        <ReferenceTitle>
            <TitleText>{}</TitleText>
        </ReferenceTitle>
        <Duration>PT{}M{}S</Duration>
        <CreationDate>2023-01-01</CreationDate>
        <LanguageOfPerformance>en</LanguageOfPerformance>
        <ResourceContributor>
            <PartyId namespace="IPI">{}</PartyId>
            <PartyName>{}</PartyName>
            <ContributorRole>MainArtist</ContributorRole>
        </ResourceContributor>
    </SoundRecording>
"#,
                resource_ref, isrc, track_title, duration_min, duration_sec, party_id, artist_name
            ));
        }

        // Add progress feedback for large files
        if i > 0 && i % 1000 == 0 {
            println!(
                "Generated {} releases, file size: {:.1}MB",
                i,
                xml.len() as f64 / (1024.0 * 1024.0)
            );
        }
    }

    xml.push_str("</ern:NewReleaseMessage>");

    println!(
        "Generated test file: {:.2}MB with {} releases",
        xml.len() as f64 / (1024.0 * 1024.0),
        target_releases
    );

    xml.into_bytes()
}

/// Profile memory allocations and string operations
fn profile_memory_allocations() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <Release ReleaseReference="MEM-TEST">
        <ReferenceTitle><TitleText>Memory Test</TitleText></ReferenceTitle>
    </Release>
</ern:NewReleaseMessage>"#;

    let iterations = 10_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let cursor = Cursor::new(xml.as_bytes());
        let iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);
        let _: Vec<_> = iterator.collect();
    }
    let elapsed = start.elapsed();

    let per_iteration = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "Memory allocation profile: {:.2}μs per parse iteration",
        per_iteration
    );

    // This reveals string allocation overhead
}

/// Profile DOM parsing bottleneck
fn profile_dom_parsing() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <Release ReleaseReference="DOM-TEST">
        <ReferenceTitle><TitleText>DOM Parsing Test</TitleText></ReferenceTitle>
    </Release>
</ern:NewReleaseMessage>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let mut iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);

    let start = Instant::now();
    let mut event_count = 0;

    while let Some(result) = iterator.next() {
        if result.is_ok() {
            event_count += 1;
        }
    }

    let elapsed = start.elapsed();
    println!(
        "DOM parsing profile: {:?} for {} events, {:.2}μs per event",
        elapsed,
        event_count,
        elapsed.as_micros() as f64 / event_count as f64
    );
}

/// Profile string operations and text extraction
fn profile_string_operations() {
    // Test string creation overhead
    let test_strings = vec![
        "Short",
        "Medium length string for testing",
        "Very long string that would be typical of a DDEX release title or other metadata field that could be quite lengthy",
    ];

    let iterations = 100_000;

    for test_str in test_strings {
        let start = Instant::now();
        for _ in 0..iterations {
            let _owned: String = test_str.to_string();
        }
        let to_string_time = start.elapsed();

        let start = Instant::now();
        for _ in 0..iterations {
            let _owned: String = test_str.to_owned();
        }
        let to_owned_time = start.elapsed();

        let start = Instant::now();
        for _ in 0..iterations {
            let _owned: String = String::from(test_str);
        }
        let from_time = start.elapsed();

        println!("String '{}' ({} bytes):", test_str, test_str.len());
        println!(
            "  to_string(): {:.2}ns per op",
            to_string_time.as_nanos() as f64 / iterations as f64
        );
        println!(
            "  to_owned():  {:.2}ns per op",
            to_owned_time.as_nanos() as f64 / iterations as f64
        );
        println!(
            "  String::from(): {:.2}ns per op",
            from_time.as_nanos() as f64 / iterations as f64
        );
    }
}

/// Comprehensive performance analysis
pub struct PerformanceAnalyzer {
    results: Vec<BenchmarkResult>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub file_size_mb: f64,
    pub throughput_mb_per_sec: f64,
    pub elements_per_sec: u64,
    pub memory_peak_mb: f64,
    pub parse_time: std::time::Duration,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn benchmark_current_implementation(&mut self) -> Result<(), ParseError> {
        println!("🔍 Benchmarking current streaming implementation...\n");

        // Test different file sizes
        let test_sizes = vec![(1, "1MB"), (10, "10MB"), (50, "50MB"), (100, "100MB")];

        for (size_mb, name) in test_sizes {
            let target_bytes = size_mb * 1024 * 1024;
            let test_data = generate_large_test_file(target_bytes);

            println!(
                "Testing {}: {:.2}MB actual size",
                name,
                test_data.len() as f64 / (1024.0 * 1024.0)
            );

            let result = self.benchmark_data(&test_data, name)?;
            self.results.push(result);
        }

        self.analyze_bottlenecks();
        Ok(())
    }

    fn benchmark_data(&self, data: &[u8], test_name: &str) -> Result<BenchmarkResult, ParseError> {
        let cursor = Cursor::new(data);
        let mut iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);

        let start = Instant::now();

        let mut element_count = 0;
        let mut max_memory = 0;

        while let Some(result) = iterator.next() {
            result?; // Propagate parse errors
            element_count += 1;

            let stats = iterator.stats();
            max_memory = max_memory.max(stats.current_memory_bytes);

            // Sample memory every 1000 elements to avoid overhead
            if element_count % 1000 == 0 {
                // Memory sampling
            }
        }

        let parse_time = start.elapsed();

        let file_size_mb = data.len() as f64 / (1024.0 * 1024.0);
        let throughput = file_size_mb / parse_time.as_secs_f64();
        let elements_per_sec = element_count as f64 / parse_time.as_secs_f64();

        let result = BenchmarkResult {
            test_name: test_name.to_string(),
            file_size_mb,
            throughput_mb_per_sec: throughput,
            elements_per_sec: elements_per_sec as u64,
            memory_peak_mb: max_memory as f64 / (1024.0 * 1024.0),
            parse_time,
        };

        println!("📊 {} Results:", test_name);
        println!("   Throughput:    {:.2} MB/s", result.throughput_mb_per_sec);
        println!("   Elements/sec:  {:.0}", result.elements_per_sec);
        println!("   Peak memory:   {:.2} MB", result.memory_peak_mb);
        println!("   Parse time:    {:.2}s", result.parse_time.as_secs_f64());
        println!();

        Ok(result)
    }

    fn analyze_bottlenecks(&self) {
        println!("🎯 BOTTLENECK ANALYSIS");
        println!("======================");

        if let Some(best) = self.results.iter().max_by(|a, b| {
            a.throughput_mb_per_sec
                .partial_cmp(&b.throughput_mb_per_sec)
                .unwrap()
        }) {
            println!(
                "Best throughput: {:.2} MB/s ({})",
                best.throughput_mb_per_sec, best.test_name
            );
        }

        let avg_throughput: f64 = self
            .results
            .iter()
            .map(|r| r.throughput_mb_per_sec)
            .sum::<f64>()
            / self.results.len() as f64;
        println!("Average throughput: {:.2} MB/s", avg_throughput);
        println!("TARGET throughput: 280 MB/s");
        println!("Required improvement: {:.0}x", 280.0 / avg_throughput);
        println!();

        println!("🚨 Identified bottlenecks:");
        if avg_throughput < 5.0 {
            println!(
                "   1. DOM-based parsing: Current implementation likely using full DOM parsing"
            );
            println!("   2. String allocations: Excessive String::clone() and to_string() calls");
            println!("   3. Memory allocations: Non-zero-copy buffer management");
            println!("   4. XML parser overhead: quick-xml may be suboptimal for streaming");
        }

        println!("\n💡 Optimization targets:");
        println!("   • Replace DOM parsing with streaming-native parsing");
        println!("   • Implement zero-copy string handling");
        println!("   • Use SIMD for pattern matching");
        println!("   • Optimize memory allocation patterns");
        println!("   • Consider specialized DDEX parsing instead of generic XML");
    }

    pub fn get_current_performance(&self) -> Option<f64> {
        self.results
            .iter()
            .map(|r| r.throughput_mb_per_sec)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_analysis() {
        let mut analyzer = PerformanceAnalyzer::new();
        analyzer.benchmark_current_implementation().unwrap();

        // Ensure we have baseline metrics
        assert!(
            !analyzer.results.is_empty(),
            "Should have benchmark results"
        );

        if let Some(current_perf) = analyzer.get_current_performance() {
            println!("Current best performance: {:.2} MB/s", current_perf);
            println!(
                "Need {:.0}x improvement to reach 280 MB/s",
                280.0 / current_perf
            );
        }
    }

    #[test]
    fn test_chunk_size_optimization() {
        let xml = generate_large_test_file(10 * 1024 * 1024); // 10MB
        let chunk_sizes = vec![1024, 8192, 65536, 1048576]; // 1KB to 1MB

        println!("Chunk size optimization analysis:");

        for chunk_size in chunk_sizes {
            let start = Instant::now();
            let mut total_elements = 0;

            for chunk in xml.chunks(chunk_size) {
                let cursor = Cursor::new(chunk);
                let iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);
                total_elements += iterator.count();
            }

            let elapsed = start.elapsed();
            let throughput = xml.len() as f64 / elapsed.as_secs_f64() / 1_000_000.0;

            println!(
                "Chunk size {:8}: {:.2} MB/s ({} elements)",
                chunk_size, throughput, total_elements
            );
        }
    }

    #[test]
    fn test_memory_allocation_profiling() {
        profile_memory_allocations();
    }

    #[test]
    fn test_dom_parsing_profiling() {
        profile_dom_parsing();
    }

    #[test]
    fn test_string_operations_profiling() {
        profile_string_operations();
    }
}
