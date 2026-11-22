//! Performance benchmark for zero-copy parser
//!
//! This benchmark compares the performance of:
//! 1. Current working streaming parser (~14.7 MB/s)
//! 2. New zero-copy parser (target: 280+ MB/s)

use crate::error::ParseError;
use crate::streaming::fast_zero_copy::FastZeroCopyIterator;
use crate::streaming::WorkingStreamIterator;
use ddex_core::models::versions::ERNVersion;
use std::io::Cursor;
use std::time::Instant;

/// Benchmark result for parser comparison
#[derive(Debug, Clone)]
pub struct BenchmarkComparison {
    pub current_parser_mb_per_sec: f64,
    pub zero_copy_parser_mb_per_sec: f64,
    pub improvement_factor: f64,
    pub file_size_mb: f64,
    pub current_elements: usize,
    pub zero_copy_elements: usize,
    pub target_achieved: bool,
}

/// Generate test XML data for benchmarking
fn generate_benchmark_xml(target_size_mb: usize) -> Vec<u8> {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>PERF-BENCHMARK-MSG</MessageId>
        <CreatedDateTime>2023-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>
"#,
    );

    let target_bytes = target_size_mb * 1024 * 1024;
    let single_release_size = 800; // Estimated size per release
    let num_releases = target_bytes / single_release_size;

    for i in 0..num_releases {
        xml.push_str(&format!(
            r#"
    <Release ReleaseReference="BENCH-REL-{:08}">
        <ReferenceTitle>
            <TitleText>Benchmark Release #{} for Performance Testing</TitleText>
            <SubTitle>Zero-Copy Performance Validation</SubTitle>
        </ReferenceTitle>
        <Genre>
            <GenreText>Electronic</GenreText>
            <SubGenre>Techno</SubGenre>
        </Genre>
        <PLine>
            <Year>2024</Year>
            <PLineText>℗ 2024 Benchmark Label</PLineText>
        </PLine>
        <ReleaseLabelReference>BENCH-LBL-{:03}</ReleaseLabelReference>
    </Release>
"#,
            i,
            i,
            i % 100
        ));

        // Add sound recordings
        for j in 0..3 {
            xml.push_str(&format!(
                r#"
    <SoundRecording ResourceReference="BENCH-RES-{:08}-{:02}">
        <ResourceId>
            <ISRC>BENCH{:010}</ISRC>
        </ResourceId>
        <ReferenceTitle>
            <TitleText>Benchmark Track {} from Release {}</TitleText>
        </ReferenceTitle>
        <Duration>PT{}M{}S</Duration>
        <CreationDate>2024-01-01</CreationDate>
        <LanguageOfPerformance>en</LanguageOfPerformance>
        <ResourceContributor>
            <PartyId namespace="IPI">BENCH{:08}</PartyId>
            <PartyName>Benchmark Artist {}</PartyName>
            <ContributorRole>MainArtist</ContributorRole>
        </ResourceContributor>
    </SoundRecording>
"#,
                i,
                j,
                i * 10 + j,
                j + 1,
                i,
                (j + 3) % 8,
                (i + j + 30) % 60,
                i,
                i % 500
            ));
        }
    }

    xml.push_str("</ern:NewReleaseMessage>");
    xml.into_bytes()
}

/// Benchmark the current working parser
fn benchmark_working_parser(data: &[u8]) -> Result<(f64, usize, std::time::Duration), ParseError> {
    let start = Instant::now();
    let cursor = Cursor::new(data);
    let mut iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);

    let mut element_count = 0;
    while let Some(result) = iterator.next() {
        result?; // Propagate errors
        element_count += 1;
    }

    let elapsed = start.elapsed();
    let throughput = (data.len() as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64();

    Ok((throughput, element_count, elapsed))
}

/// Benchmark the fast zero-copy parser
fn benchmark_zero_copy_parser(
    data: &[u8],
) -> Result<(f64, usize, std::time::Duration), ParseError> {
    let start = Instant::now();
    let cursor = Cursor::new(data);
    let mut iterator = FastZeroCopyIterator::new(cursor, ERNVersion::V4_3);

    let mut element_count = 0;
    while let Some(result) = iterator.next() {
        result?; // Propagate errors
        element_count += 1;
    }

    let elapsed = start.elapsed();
    let throughput = (data.len() as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64();

    Ok((throughput, element_count, elapsed))
}

/// Run comprehensive performance comparison
pub fn run_performance_benchmark() -> Result<Vec<BenchmarkComparison>, ParseError> {
    println!("🚀 Running Performance Benchmark: Zero-Copy vs Working Parser");
    println!("================================================================");

    let test_sizes = vec![1, 5, 10, 25, 50]; // MB
    let mut results = Vec::new();

    for size_mb in test_sizes {
        println!("\n🔬 Testing {size_mb}MB file...");

        // Generate test data
        let test_data = generate_benchmark_xml(size_mb);
        let actual_size_mb = test_data.len() as f64 / (1024.0 * 1024.0);
        println!("   Generated: {:.2}MB", actual_size_mb);

        // Benchmark working parser
        print!("   Working parser:  ");
        let (current_throughput, current_elements, current_time) =
            benchmark_working_parser(&test_data)?;
        println!(
            "{:.2} MB/s ({} elements, {:.3}s)",
            current_throughput,
            current_elements,
            current_time.as_secs_f64()
        );

        // Benchmark zero-copy parser
        print!("   Zero-copy parser:");
        let (zero_copy_throughput, zero_copy_elements, zero_copy_time) =
            benchmark_zero_copy_parser(&test_data)?;
        println!(
            "{:.2} MB/s ({} elements, {:.3}s)",
            zero_copy_throughput,
            zero_copy_elements,
            zero_copy_time.as_secs_f64()
        );

        // Calculate improvement
        let improvement = zero_copy_throughput / current_throughput;
        let target_achieved = zero_copy_throughput >= 280.0;

        println!("   📈 Improvement:   {:.1}x faster", improvement);
        println!(
            "   🎯 Target 280MB/s: {}",
            if target_achieved {
                "✅ ACHIEVED"
            } else {
                "❌ Not yet"
            }
        );

        results.push(BenchmarkComparison {
            current_parser_mb_per_sec: current_throughput,
            zero_copy_parser_mb_per_sec: zero_copy_throughput,
            improvement_factor: improvement,
            file_size_mb: actual_size_mb,
            current_elements: current_elements,
            zero_copy_elements: zero_copy_elements,
            target_achieved,
        });
    }

    // Summary
    println!("\n📊 BENCHMARK SUMMARY");
    println!("====================");

    let avg_current = results
        .iter()
        .map(|r| r.current_parser_mb_per_sec)
        .sum::<f64>()
        / results.len() as f64;
    let avg_zero_copy = results
        .iter()
        .map(|r| r.zero_copy_parser_mb_per_sec)
        .sum::<f64>()
        / results.len() as f64;
    let avg_improvement = avg_zero_copy / avg_current;

    println!("Average Throughput:");
    println!("  Working Parser:  {:.2} MB/s", avg_current);
    println!("  Zero-Copy Parser: {:.2} MB/s", avg_zero_copy);
    println!("  Average Improvement: {:.1}x", avg_improvement);
    println!();

    let targets_met = results.iter().filter(|r| r.target_achieved).count();
    if targets_met == results.len() {
        println!("🎉 SUCCESS: All target performance goals achieved!");
    } else if targets_met > 0 {
        println!(
            "🔥 PARTIAL SUCCESS: {}/{} test sizes achieved 280+ MB/s target",
            targets_met,
            results.len()
        );
    } else {
        println!("⚠️  TARGET NOT MET: Zero-copy parser needs more optimization");
    }

    // Performance requirement analysis
    let original_target_improvement = 480.0; // From 0.58 MB/s to 280 MB/s
    let current_baseline = 14.7; // Current working parser baseline
    let required_improvement = 280.0 / current_baseline;

    println!("\n📈 PERFORMANCE ANALYSIS");
    println!("========================");
    println!(
        "Original goal: {:.0}x improvement (0.58 → 280 MB/s)",
        original_target_improvement
    );
    println!(
        "Revised goal:  {:.1}x improvement ({:.1} → 280 MB/s)",
        required_improvement, current_baseline
    );
    println!(
        "Achieved:      {:.1}x improvement ({:.1} → {:.1} MB/s)",
        avg_improvement, avg_current, avg_zero_copy
    );

    if avg_zero_copy >= 280.0 {
        println!("✅ MISSION ACCOMPLISHED: 480x performance target achieved!");
    } else {
        let remaining = 280.0 / avg_zero_copy;
        println!(
            "🔧 Additional {:.1}x improvement needed to reach target",
            remaining
        );
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_comparison() {
        let results = run_performance_benchmark().unwrap();
        assert!(!results.is_empty(), "Should have benchmark results");

        for result in &results {
            assert!(
                result.zero_copy_parser_mb_per_sec > 0.0,
                "Zero-copy parser should have positive throughput"
            );
            assert!(
                result.current_parser_mb_per_sec > 0.0,
                "Working parser should have positive throughput"
            );
            assert!(
                result.improvement_factor > 0.0,
                "Should have positive improvement factor"
            );

            // Zero-copy parser should be faster (even if not 480x faster)
            assert!(
                result.zero_copy_parser_mb_per_sec > result.current_parser_mb_per_sec,
                "Zero-copy parser should be faster than working parser"
            );
        }

        println!("Benchmark validated: Zero-copy parser shows performance improvement");
    }

    #[test]
    fn test_xml_generation() {
        let xml = generate_benchmark_xml(1);
        let xml_str = String::from_utf8_lossy(&xml);

        assert!(
            xml_str.contains("MessageHeader"),
            "Should have message header"
        );
        assert!(xml_str.contains("Release"), "Should have releases");
        assert!(
            xml_str.contains("SoundRecording"),
            "Should have sound recordings"
        );
        assert!(
            xml.len() >= 1024 * 1024 / 2,
            "Should be reasonably sized for 1MB target"
        );
    }
}
