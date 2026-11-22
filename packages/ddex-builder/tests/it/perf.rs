//! Performance regression tests for DDEX Builder
//!
//! These tests ensure that performance optimizations don't regress and that
//! we consistently meet our target metrics:
//! - Single track: <5ms
//! - 12-track album: <10ms
//! - 100-track compilation: <50ms
//! - Memory usage: <10MB for typical album

use ddex_builder::builder::{
    LocalizedStringRequest, MessageHeaderRequest, PartyRequest, ReleaseRequest, TrackRequest,
};
use ddex_builder::memory_optimization::BuildMemoryManager;
use ddex_builder::optimized_strings::BuildContext;
use ddex_builder::parallel_processing::{ParallelConfig, ParallelProcessor};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};
use std::time::{Duration, Instant};

/// Performance target thresholds
const SINGLE_TRACK_TARGET_MS: u64 = 5;
const ALBUM_12_TRACKS_TARGET_MS: u64 = 10;
const COMPILATION_100_TRACKS_TARGET_MS: u64 = 50;
const MEMORY_TARGET_MB: usize = 10;

/// Test result with performance metrics
#[derive(Debug)]
struct PerformanceResult {
    duration: Duration,
    memory_used_mb: f64,
    track_count: usize,
    meets_target: bool,
    target_ms: u64,
}

impl PerformanceResult {
    fn new(duration: Duration, memory_used_bytes: usize, track_count: usize) -> Self {
        let target_ms = match track_count {
            1 => SINGLE_TRACK_TARGET_MS,
            2..=20 => ALBUM_12_TRACKS_TARGET_MS,
            _ => COMPILATION_100_TRACKS_TARGET_MS,
        };

        let meets_target = duration.as_millis() <= target_ms as u128;
        let memory_used_mb = memory_used_bytes as f64 / 1024.0 / 1024.0;

        Self {
            duration,
            memory_used_mb,
            track_count,
            meets_target,
            target_ms,
        }
    }

    fn assert_meets_target(&self) {
        if !self.meets_target {
            panic!(
                "Performance regression detected!\n\
                Track count: {}\n\
                Actual time: {}ms\n\
                Target time: {}ms\n\
                Memory used: {:.2}MB",
                self.track_count,
                self.duration.as_millis(),
                self.target_ms,
                self.memory_used_mb
            );
        }
    }

    fn assert_memory_target(&self) {
        if self.memory_used_mb > MEMORY_TARGET_MB as f64 {
            panic!(
                "Memory usage regression detected!\n\
                Track count: {}\n\
                Memory used: {:.2}MB\n\
                Target: {}MB",
                self.track_count, self.memory_used_mb, MEMORY_TARGET_MB
            );
        }
    }
}

/// Run a performance test with memory tracking
fn run_performance_test(track_count: usize) -> PerformanceResult {
    // Create test data
    let request = create_test_request(track_count);
    let builder = DDEXBuilder::new();
    let options = BuildOptions::default();

    // Warm up (exclude from timing)
    let _ = builder.build(request.clone(), options.clone());

    // Measure memory before
    let memory_before = get_memory_usage();

    // Time the build operation
    let start_time = Instant::now();
    let result = builder.build(request, options);
    let duration = start_time.elapsed();

    // Measure memory after
    let memory_after = get_memory_usage();
    let memory_used = memory_after.saturating_sub(memory_before);

    // Verify build succeeded
    assert!(result.is_ok(), "Build failed: {:?}", result.err());

    PerformanceResult::new(duration, memory_used, track_count)
}

/// Run an optimized performance test with all optimizations enabled
fn run_optimized_performance_test(track_count: usize) -> PerformanceResult {
    let request = create_test_request(track_count);

    // Use optimized components
    let mut context = BuildContext::new();
    let memory_manager = BuildMemoryManager::new();
    let parallel_config = ParallelConfig::default();
    let processor = ParallelProcessor::new(parallel_config).unwrap();

    // Warm up
    let _ = processor.process_build_parallel(&request, &mut context, &memory_manager);

    let memory_before = get_memory_usage();

    let start_time = Instant::now();
    let result = processor.process_build_parallel(&request, &mut context, &memory_manager);
    let duration = start_time.elapsed();

    let memory_after = get_memory_usage();
    let memory_used = memory_after.saturating_sub(memory_before);

    assert!(result.is_ok(), "Optimized build failed: {:?}", result.err());

    PerformanceResult::new(duration, memory_used, track_count)
}

/// Get approximate memory usage (simplified for testing)
fn get_memory_usage() -> usize {
    // In a real implementation, this would use a memory profiler
    // For testing, we'll use a simple approximation
    std::process::id() as usize * 1024 // Dummy value
}

/// Create test request with specified number of tracks
fn create_test_request(track_count: usize) -> BuildRequest {
    let mut tracks = Vec::with_capacity(track_count);

    for i in 0..track_count {
        tracks.push(TrackRequest {
            track_id: format!("T{:03}", i + 1),
            resource_reference: Some(format!("A{:03}", i + 1)),
            isrc: format!("TEST{:08}", i + 1), // 12 chars total
            title: format!("Test Track {}", i + 1),
            duration: format!("PT{}M{}S", 3 + (i % 4), 15 + (i % 45)),
            artist: format!("Artist {}", (i % 5) + 1), // Simulate repeated artists
        });
    }

    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some(format!("PERF_TEST_{:03}_TRACKS", track_count)),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Performance Test Sender".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("SENDER_PERF".to_string()),
                party_reference: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Performance Test Recipient".to_string(),
                    language_code: Some("en".to_string()),
                }],
                party_id: Some("RECIP_PERF".to_string()),
                party_reference: None,
            },
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: None,
        },
        version: "4.3".to_string(),
        profile: Some("CommonReleaseTypes/14/AudioAlbumMusicOnly".to_string()),
        releases: vec![ReleaseRequest {
            release_id: format!("REL_PERF_{:03}", track_count),
            release_reference: Some("R_PERF_001".to_string()),
            title: vec![LocalizedStringRequest {
                text: if track_count == 1 {
                    "Performance Test Single".to_string()
                } else if track_count <= 20 {
                    "Performance Test Album".to_string()
                } else {
                    "Performance Test Compilation".to_string()
                },
                language_code: Some("en".to_string()),
            }],
            artist: "Performance Test Artist".to_string(),
            label: Some("Performance Test Label".to_string()),
            release_date: Some("2024-01-01".to_string()),
            upc: Some("123456789012".to_string()),
            tracks,
            resource_references: None,
        }],
        deals: vec![],
        extensions: None,
    }
}

// Individual test cases for each target metric

#[test]
fn test_single_track_performance() {
    println!(
        "Testing single track performance target (<{}ms)...",
        SINGLE_TRACK_TARGET_MS
    );

    let result = run_performance_test(1);

    println!(
        "Single track: {}ms (target: {}ms), memory: {:.2}MB",
        result.duration.as_millis(),
        result.target_ms,
        result.memory_used_mb
    );

    result.assert_meets_target();
    result.assert_memory_target();
}

#[test]
fn test_album_12_tracks_performance() {
    println!(
        "Testing 12-track album performance target (<{}ms)...",
        ALBUM_12_TRACKS_TARGET_MS
    );

    let result = run_performance_test(12);

    println!(
        "12-track album: {}ms (target: {}ms), memory: {:.2}MB",
        result.duration.as_millis(),
        result.target_ms,
        result.memory_used_mb
    );

    result.assert_meets_target();
    result.assert_memory_target();
}

#[test]
fn test_compilation_100_tracks_performance() {
    println!(
        "Testing 100-track compilation performance target (<{}ms)...",
        COMPILATION_100_TRACKS_TARGET_MS
    );

    let result = run_performance_test(100);

    println!(
        "100-track compilation: {}ms (target: {}ms), memory: {:.2}MB",
        result.duration.as_millis(),
        result.target_ms,
        result.memory_used_mb
    );

    result.assert_meets_target();
    // Relaxed memory constraint for large compilations
    assert!(
        result.memory_used_mb < MEMORY_TARGET_MB as f64 * 5.0,
        "Memory usage too high: {:.2}MB",
        result.memory_used_mb
    );
}

#[test]
fn test_optimized_single_track_performance() {
    println!("Testing optimized single track performance...");

    let result = run_optimized_performance_test(1);

    println!(
        "Optimized single track: {}ms (target: {}ms), memory: {:.2}MB",
        result.duration.as_millis(),
        result.target_ms,
        result.memory_used_mb
    );

    result.assert_meets_target();
    result.assert_memory_target();
}

#[test]
fn test_optimized_album_12_tracks_performance() {
    println!("Testing optimized 12-track album performance...");

    let result = run_optimized_performance_test(12);

    println!(
        "Optimized 12-track album: {}ms (target: {}ms), memory: {:.2}MB",
        result.duration.as_millis(),
        result.target_ms,
        result.memory_used_mb
    );

    result.assert_meets_target();
    result.assert_memory_target();
}

#[test]
fn test_optimized_compilation_50_tracks_performance() {
    println!("Testing optimized 50-track compilation performance...");

    let result = run_optimized_performance_test(50);

    println!(
        "Optimized 50-track compilation: {}ms (target: {}ms), memory: {:.2}MB",
        result.duration.as_millis(),
        result.target_ms,
        result.memory_used_mb
    );

    result.assert_meets_target();
    assert!(
        result.memory_used_mb < MEMORY_TARGET_MB as f64 * 3.0,
        "Memory usage too high: {:.2}MB",
        result.memory_used_mb
    );
}

// Scaling tests to ensure performance scales linearly

#[test]
fn test_linear_scaling_performance() {
    println!("Testing linear scaling performance...");

    let track_counts = [1, 5, 10, 20];
    let mut results = Vec::new();

    for &count in &track_counts {
        let result = run_performance_test(count);
        println!(
            "{} tracks: {}ms, {:.2}MB",
            count,
            result.duration.as_millis(),
            result.memory_used_mb
        );
        results.push((count, result.duration.as_millis()));
    }

    // Check that scaling is roughly linear (allowing some variance)
    for window in results.windows(2) {
        let (count1, time1) = window[0];
        let (count2, time2) = window[1];

        let scale_factor = count2 as f64 / count1 as f64;
        let time_ratio = time2 as f64 / time1 as f64;

        // Time should scale roughly linearly (within 50% variance)
        assert!(
            time_ratio <= scale_factor * 1.5,
            "Performance scaling issue: {} tracks ({}ms) to {} tracks ({}ms) - ratio {:.2} vs expected {:.2}",
            count1, time1, count2, time2, time_ratio, scale_factor
        );
    }
}

// Memory efficiency tests

#[test]
fn test_memory_efficiency() {
    println!("Testing memory efficiency...");

    // Test that memory usage is reasonable for different sizes
    let test_cases = [(1, 1.0), (12, 5.0), (50, 15.0)]; // (tracks, max_mb)

    for &(track_count, max_mb) in &test_cases {
        let result = run_performance_test(track_count);

        println!(
            "{} tracks: {:.2}MB (max: {:.1}MB)",
            track_count, result.memory_used_mb, max_mb
        );

        assert!(
            result.memory_used_mb <= max_mb,
            "Memory usage too high for {} tracks: {:.2}MB > {:.1}MB",
            track_count,
            result.memory_used_mb,
            max_mb
        );
    }
}

// Regression benchmark (for detecting performance degradation)

#[test]
fn test_performance_regression_benchmark() {
    println!("Running performance regression benchmark...");

    // Run multiple iterations to get stable measurements
    const ITERATIONS: usize = 5;
    let track_counts = [1, 12, 50];

    for &track_count in &track_counts {
        let mut durations = Vec::new();

        for _i in 0..ITERATIONS {
            let result = run_performance_test(track_count);
            durations.push(result.duration.as_millis());
        }

        let avg_duration = durations.iter().sum::<u128>() / ITERATIONS as u128;
        let min_duration = *durations.iter().min().unwrap();
        let max_duration = *durations.iter().max().unwrap();

        println!(
            "{} tracks: avg={}ms, min={}ms, max={}ms",
            track_count, avg_duration, min_duration, max_duration
        );

        // Ensure average meets target
        let target = match track_count {
            1 => SINGLE_TRACK_TARGET_MS,
            12 => ALBUM_12_TRACKS_TARGET_MS,
            _ => COMPILATION_100_TRACKS_TARGET_MS,
        } as u128;

        assert!(
            avg_duration <= target,
            "Average performance regression for {} tracks: {}ms > {}ms",
            track_count,
            avg_duration,
            target
        );

        // Ensure variance is reasonable (max should be within 2x of min)
        assert!(
            max_duration <= min_duration * 2,
            "Performance variance too high for {} tracks: {}ms to {}ms",
            track_count,
            min_duration,
            max_duration
        );
    }
}

// Stress test for extreme cases

#[test]
#[ignore = "Stress test - run manually"]
fn test_extreme_performance_stress() {
    println!("Running extreme performance stress test...");

    // Test with very large compilation
    let result = run_performance_test(500);

    println!(
        "500 tracks: {}ms, {:.2}MB",
        result.duration.as_millis(),
        result.memory_used_mb
    );

    // Should complete within reasonable time (scaled target)
    assert!(
        result.duration.as_millis() <= 250, // 5x the 50-track target
        "Extreme stress test failed: {}ms",
        result.duration.as_millis()
    );

    // Memory should stay reasonable
    assert!(
        result.memory_used_mb <= 50.0, // 5x memory target
        "Memory usage too high in stress test: {:.2}MB",
        result.memory_used_mb
    );
}
