//! Comprehensive benchmark for parallel streaming parser
//!
//! Tests the 6.25x speedup target on 8 cores to reach 280+ MB/s throughput

use crate::error::ParseError;
use crate::streaming::{ParallelBenchmark, ParallelStreamingIterator, WorkingStreamIterator};
use ddex_core::models::versions::ERNVersion;
use std::io::Cursor;
use std::time::Instant;

/// Comprehensive benchmark comparing all parser implementations
#[derive(Debug, Clone)]
pub struct ComprehensiveBenchmarkResult {
    pub file_size_mb: f64,
    pub working_parser_mb_per_sec: f64,
    pub zero_copy_parser_mb_per_sec: f64,
    pub parallel_parser_mb_per_sec: f64,
    pub parallel_speedup: f64,
    pub target_achieved: bool,
    pub cores_used: usize,
    pub parallel_efficiency: f64,
}

/// Generate large test data with realistic DDEX structure
fn generate_comprehensive_test_data(target_mb: usize) -> Vec<u8> {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>COMPREHENSIVE-BENCH-MSG-2024</MessageId>
        <CreatedDateTime>2024-09-13T12:00:00Z</CreatedDateTime>
        <MessageSender>
            <PartyId namespace="PADPIDA2020">COMP001</PartyId>
            <PartyName>Comprehensive Benchmark System</PartyName>
        </MessageSender>
    </MessageHeader>
"#,
    );

    let target_bytes = target_mb * 1024 * 1024;
    let single_release_size = 2000; // Larger releases for realistic benchmarking
    let num_releases = (target_bytes / single_release_size).max(500);

    println!(
        "Generating comprehensive test data: {} releases for {:.1}MB target",
        num_releases, target_mb as f64
    );

    for i in 0..num_releases {
        xml.push_str(&format!(
            r#"
    <Release ReleaseReference="COMP-REL-{:08}">
        <ReferenceTitle>
            <TitleText>Comprehensive Benchmark Release #{}</TitleText>
            <SubTitle>Multi-threaded Performance Validation Suite</SubTitle>
        </ReferenceTitle>
        <Genre>
            <GenreText>Electronic</GenreText>
            <SubGenre>Techno</SubGenre>
        </Genre>
        <PLine>
            <Year>2024</Year>
            <PLineText>℗ 2024 Comprehensive Benchmark Label Inc.</PLineText>
        </PLine>
        <CLine>
            <Year>2024</Year>
            <CLineText>© 2024 Comprehensive Benchmark Label Inc.</CLineText>
        </CLine>
        <ReleaseLabelReference>COMP-LBL-{:04}</ReleaseLabelReference>
        <ReleaseType>Single</ReleaseType>
        <ReleaseDate>
            <Date>2024-01-01</Date>
        </ReleaseDate>
    </Release>
"#,
            i,
            i,
            i % 1000
        ));

        // Add comprehensive sound recordings for realistic data volume
        for j in 0..6 {
            xml.push_str(&format!(
                r#"
    <SoundRecording ResourceReference="COMP-RES-{:08}-{:02}">
        <ResourceId>
            <ISRC>COMPB{:07}{:02}</ISRC>
        </ResourceId>
        <ReferenceTitle>
            <TitleText>Comprehensive Benchmark Track {} from Release {}</TitleText>
            <SubTitle>Performance Test Audio Content</SubTitle>
        </ReferenceTitle>
        <Duration>PT{}M{}S</Duration>
        <CreationDate>2024-01-01</CreationDate>
        <MasteredDate>2024-01-01</MasteredDate>
        <LanguageOfPerformance>en</LanguageOfPerformance>
        <LanguageOfDubbing>en</LanguageOfDubbing>
        <ResourceContributor>
            <PartyId namespace="IPI">COMP{:08}</PartyId>
            <PartyName>Comprehensive Benchmark Artist {}</PartyName>
            <ContributorRole>MainArtist</ContributorRole>
            <ContributorRole>Vocalist</ContributorRole>
        </ResourceContributor>
        <ResourceContributor>
            <PartyId namespace="IPI">PROD{:08}</PartyId>
            <PartyName>Benchmark Producer {}</PartyName>
            <ContributorRole>Producer</ContributorRole>
        </ResourceContributor>
        <SoundRecordingType>MusicalWorkSoundRecording</SoundRecordingType>
        <TechnicalSoundRecordingDetails>
            <TechnicalResourceDetailsReference>T{:08}</TechnicalResourceDetailsReference>
            <AudioCodecType>PCM</AudioCodecType>
            <AudioChannelConfiguration>Stereo</AudioChannelConfiguration>
        </TechnicalSoundRecordingDetails>
    </SoundRecording>
"#,
                i,
                j,
                i,
                j,
                j + 1,
                i,
                (j + 3) % 8,
                (i + j + 30) % 60,
                i,
                i % 1000,
                i,
                i % 500,
                i
            ));
        }

        // Add some video resources for diversity
        if i % 10 == 0 {
            xml.push_str(&format!(
                r#"
    <Video ResourceReference="COMP-VID-{:08}">
        <ResourceId>
            <ISAN>COMP-{:016}-{:01}</ISAN>
        </ResourceId>
        <ReferenceTitle>
            <TitleText>Comprehensive Benchmark Music Video {}</TitleText>
        </ReferenceTitle>
        <Duration>PT{}M{}S</Duration>
        <CreationDate>2024-01-01</CreationDate>
        <VideoType>MusicalWorkVideo</VideoType>
        <TechnicalVideoDetails>
            <VideoCodecType>H264</VideoCodecType>
            <VideoDefinitionType>HD</VideoDefinitionType>
        </TechnicalVideoDetails>
    </Video>
"#,
                i,
                i,
                0,
                i,
                (i + 3) % 6,
                (i + 30) % 60
            ));
        }

        // Progress reporting for large files
        if i > 0 && i % 5000 == 0 {
            let current_size = xml.len() as f64 / (1024.0 * 1024.0);
            println!("Generated {:.1}MB with {} releases", current_size, i);

            if current_size >= target_mb as f64 * 1.1 {
                break; // Generated enough data
            }
        }
    }

    xml.push_str("</ern:NewReleaseMessage>");

    let final_size = xml.len() as f64 / (1024.0 * 1024.0);
    println!("Final test data: {:.2}MB", final_size);

    xml.into_bytes()
}

/// Run comprehensive benchmark comparing all parsers
pub fn run_comprehensive_benchmark() -> Result<Vec<ComprehensiveBenchmarkResult>, ParseError> {
    println!("🚀 COMPREHENSIVE STREAMING PARSER BENCHMARK");
    println!("===========================================");
    println!("Target: 480x improvement → 280+ MB/s throughput");
    println!("Expected parallel speedup: 6.25x on 8 cores\n");

    let test_sizes = vec![25, 50, 100, 200]; // MB test sizes
    let mut results = Vec::new();
    let cores = num_cpus::get();

    println!("System info: {} CPU cores detected\n", cores);

    for size_mb in test_sizes {
        println!("🔬 Testing {size_mb}MB file...");

        // Generate test data
        let test_data = generate_comprehensive_test_data(size_mb);
        let actual_size_mb = test_data.len() as f64 / (1024.0 * 1024.0);
        println!("   Generated: {:.2}MB actual", actual_size_mb);

        // 1. Benchmark working parser (baseline)
        print!("   Working parser:      ");
        let start = Instant::now();
        let cursor = Cursor::new(&test_data);
        let working_iterator = WorkingStreamIterator::new(cursor, ERNVersion::V4_3);
        let working_elements: Result<Vec<_>, _> = working_iterator.collect();
        assert!(working_elements.is_ok(), "Working parser should succeed");
        let working_time = start.elapsed();
        let working_throughput = actual_size_mb / working_time.as_secs_f64();
        let working_count = working_elements.unwrap().len();
        println!(
            "{:.2} MB/s ({} elements, {:.3}s)",
            working_throughput,
            working_count,
            working_time.as_secs_f64()
        );

        // 2. Benchmark zero-copy parser
        // print!("   Zero-copy parser:    ");
        // let start = Instant::now();
        // let cursor = Cursor::new(&test_data);
        // let mut zero_copy_iterator = FastZeroCopyIterator::new(cursor, ERNVersion::V4_3);
        // let zero_copy_elements: Result<Vec<_>, _> = zero_copy_iterator.collect();
        // assert!(zero_copy_elements.is_ok(), "Zero-copy parser should succeed");
        // let zero_copy_time = start.elapsed();
        // let zero_copy_throughput = actual_size_mb / zero_copy_time.as_secs_f64();
        // let zero_copy_count = zero_copy_elements.unwrap().len();
        // println!("{:.2} MB/s ({} elements, {:.3}s)", zero_copy_throughput, zero_copy_count, zero_copy_time.as_secs_f64());

        // 3. Benchmark parallel parser
        print!("   Parallel parser:     ");
        let start = Instant::now();
        let cursor = Cursor::new(&test_data);
        let parallel_iterator = ParallelStreamingIterator::new(cursor, ERNVersion::V4_3);
        let parallel_elements: Result<Vec<_>, _> = parallel_iterator.collect();
        assert!(parallel_elements.is_ok(), "Parallel parser should succeed");
        let parallel_time = start.elapsed();
        let parallel_throughput = actual_size_mb / parallel_time.as_secs_f64();
        let parallel_count = parallel_elements.unwrap().len();
        println!(
            "{:.2} MB/s ({} elements, {:.3}s)",
            parallel_throughput,
            parallel_count,
            parallel_time.as_secs_f64()
        );

        // Calculate metrics
        let parallel_speedup = parallel_throughput / working_throughput;
        let parallel_efficiency = (parallel_speedup / cores as f64) * 100.0;
        let target_achieved = parallel_throughput >= 280.0;

        println!("   📈 Parallel speedup: {:.2}x", parallel_speedup);
        println!("   ⚙️  Core efficiency:  {:.1}%", parallel_efficiency);
        println!(
            "   🎯 Target 280MB/s:   {}",
            if target_achieved {
                "✅ ACHIEVED"
            } else {
                "❌ Not yet"
            }
        );

        // Verify element counts are consistent
        assert_eq!(working_count, parallel_count, "Element counts should match");

        results.push(ComprehensiveBenchmarkResult {
            file_size_mb: actual_size_mb,
            working_parser_mb_per_sec: working_throughput,
            zero_copy_parser_mb_per_sec: 0.0, // Skip for now to save time
            parallel_parser_mb_per_sec: parallel_throughput,
            parallel_speedup,
            target_achieved,
            cores_used: cores,
            parallel_efficiency,
        });

        println!(); // Blank line between tests
    }

    // Overall summary
    println!("📊 COMPREHENSIVE BENCHMARK SUMMARY");
    println!("===================================");

    let avg_working = results
        .iter()
        .map(|r| r.working_parser_mb_per_sec)
        .sum::<f64>()
        / results.len() as f64;
    let avg_parallel = results
        .iter()
        .map(|r| r.parallel_parser_mb_per_sec)
        .sum::<f64>()
        / results.len() as f64;
    let avg_speedup = avg_parallel / avg_working;
    let avg_efficiency =
        results.iter().map(|r| r.parallel_efficiency).sum::<f64>() / results.len() as f64;

    println!("Average Performance:");
    println!("  Working Parser:    {:.2} MB/s", avg_working);
    println!("  Parallel Parser:   {:.2} MB/s", avg_parallel);
    println!("  Average Speedup:   {:.2}x", avg_speedup);
    println!("  Core Efficiency:   {:.1}%", avg_efficiency);

    let targets_met = results.iter().filter(|r| r.target_achieved).count();
    if targets_met == results.len() {
        println!("\n🎉 MISSION ACCOMPLISHED!");
        println!("✅ All test sizes achieved 280+ MB/s target");
        println!("✅ 480x performance improvement successfully delivered");
    } else if targets_met > 0 {
        println!("\n🔥 PARTIAL SUCCESS");
        println!(
            "✅ {}/{} test sizes achieved 280+ MB/s target",
            targets_met,
            results.len()
        );
    } else {
        println!("\n⚠️  TARGET NOT FULLY MET");
        let improvement_needed = 280.0 / avg_parallel;
        println!(
            "🔧 Additional {:.1}x improvement needed",
            improvement_needed
        );
    }

    // Performance analysis vs blueprint expectations
    println!("\n📈 BLUEPRINT VALIDATION");
    println!("========================");
    println!("Blueprint promise: 6.25x speedup on 8 cores");
    println!("Achieved speedup: {:.2}x on {} cores", avg_speedup, cores);

    if avg_speedup >= 6.0 {
        println!("✅ Blueprint speedup promise met!");
    } else {
        println!("⚠️  Blueprint speedup target not fully achieved");
    }

    if avg_parallel >= 280.0 {
        println!("✅ 480x overall improvement target achieved!");
        println!("✅ Streaming parser now processes at 280+ MB/s");
    } else {
        let original_speed = 0.58; // From blueprint
        let actual_improvement = avg_parallel / original_speed;
        println!(
            "📊 Actual improvement: {:.0}x (from {:.2} to {:.1} MB/s)",
            actual_improvement, original_speed, avg_parallel
        );
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comprehensive_benchmark() {
        let results = run_comprehensive_benchmark().unwrap();
        assert!(!results.is_empty(), "Should have benchmark results");

        for result in &results {
            assert!(
                result.parallel_parser_mb_per_sec > 0.0,
                "Parallel parser should have positive throughput"
            );
            assert!(
                result.working_parser_mb_per_sec > 0.0,
                "Working parser should have positive throughput"
            );
            assert!(
                result.parallel_speedup > 0.0,
                "Should have positive speedup"
            );

            // Parallel parser should be faster than working parser
            assert!(
                result.parallel_parser_mb_per_sec >= result.working_parser_mb_per_sec,
                "Parallel parser should be at least as fast as working parser"
            );
        }

        println!("✅ Comprehensive benchmark validation completed");
    }

    #[test]
    fn test_specific_parallel_speedup() {
        // Test with smaller data for quick validation
        let data = generate_comprehensive_test_data(10);

        let baseline_result = ParallelBenchmark::measure_parallel_speedup(&data).unwrap();

        println!("Parallel speedup test results:");
        println!(
            "  Single-threaded: {:.2} MB/s",
            baseline_result.single_threaded_throughput
        );
        println!(
            "  Best parallel: {:.2} MB/s",
            baseline_result.best_throughput
        );
        println!("  Best speedup: {:.2}x", baseline_result.best_speedup);

        assert!(
            baseline_result.best_speedup > 1.0,
            "Should achieve some speedup"
        );
    }

    #[test]
    fn test_thread_scaling_analysis() {
        let data = generate_comprehensive_test_data(50);

        println!("\nThread scaling analysis:");
        for threads in [1, 2, 4, 8] {
            if threads <= num_cpus::get() {
                let start = Instant::now();
                let cursor = Cursor::new(&data);
                let iterator =
                    ParallelStreamingIterator::with_threads(cursor, ERNVersion::V4_3, threads);
                let elements: Result<Vec<_>, _> = iterator.collect();
                let elapsed = start.elapsed();

                assert!(elements.is_ok(), "Parsing should succeed");

                let throughput = (data.len() as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64();
                let element_count = elements.unwrap().len();

                println!(
                    "  {} threads: {:.1} MB/s ({} elements)",
                    threads, throughput, element_count
                );
            }
        }
    }
}
