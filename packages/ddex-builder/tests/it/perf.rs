//! Sanity limits against catastrophic slowdowns, about 10× the timings recorded in
//! `dev-docs/PERFORMANCE.md`. They are not performance targets: the numbers come from the
//! benches (`cargo bench -p ddex-builder --bench performance`). Release mode only:
//!
//!     cargo test -p ddex-builder --release --features perf-tests perf::

#[path = "../../benches/common/mod.rs"]
mod bench_inputs;

use ddex_builder::{BuildOptions, DDEXBuilder};
use std::time::{Duration, Instant};

/// Median wall time of building a `track_count`-track release (after one warm-up build).
fn median_build_time(track_count: usize, runs: usize) -> Duration {
    let builder = DDEXBuilder::new();
    let request = bench_inputs::request_with_tracks(track_count);
    let build = || {
        builder
            .build(request.clone(), BuildOptions::default())
            .expect("the bench request must build")
    };
    let xml = build().xml;
    assert_eq!(xml.matches("<SoundRecording>").count(), track_count);
    let mut times: Vec<Duration> = (0..runs)
        .map(|_| {
            let start = Instant::now();
            build();
            start.elapsed()
        })
        .collect();
    times.sort();
    times[runs / 2]
}

fn check(track_count: usize, recorded: Duration) {
    let limit = recorded * 10;
    let median = median_build_time(track_count, 21);
    println!("{track_count}-track build: median {median:?} (limit {limit:?})");
    assert!(
        median < limit,
        "{track_count}-track build took {median:?}, sanity limit {limit:?}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "timing sanity limit: release mode only")]
fn single_track_builds_within_sanity_limit() {
    check(1, Duration::from_micros(RECORDED_1_TRACK_US));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "timing sanity limit: release mode only")]
fn twelve_track_album_builds_within_sanity_limit() {
    check(12, Duration::from_micros(RECORDED_12_TRACKS_US));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "timing sanity limit: release mode only")]
fn hundred_track_compilation_builds_within_sanity_limit() {
    check(100, Duration::from_micros(RECORDED_100_TRACKS_US));
}

// Medians from `cargo bench -p ddex-builder --bench performance` (dev-docs/PERFORMANCE.md).
const RECORDED_1_TRACK_US: u64 = 2_800;
const RECORDED_12_TRACKS_US: u64 = 11_500;
const RECORDED_100_TRACKS_US: u64 = 81_000;
