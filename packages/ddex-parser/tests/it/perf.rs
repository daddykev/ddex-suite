//! Sanity limits against catastrophic slowdowns, about 10× the timings recorded in
//! `dev-docs/PERFORMANCE.md`. They are not performance targets: the numbers come from the
//! benches (`cargo bench -p ddex-parser --bench throughput`). Release mode only:
//!
//!     cargo test -p ddex-parser --release --features perf-tests perf::

#[path = "../../benches/common/mod.rs"]
mod bench_inputs;

use ddex_parser::DDEXParser;
use std::io::Cursor;
use std::time::{Duration, Instant};

/// Median wall time of `runs` calls (after one warm-up call).
fn median_time(runs: usize, mut f: impl FnMut()) -> Duration {
    f();
    let mut times: Vec<Duration> = (0..runs)
        .map(|_| {
            let start = Instant::now();
            f();
            start.elapsed()
        })
        .collect();
    times.sort();
    times[runs / 2]
}

#[test]
#[cfg_attr(debug_assertions, ignore = "timing sanity limit: release mode only")]
fn typical_album_parses_within_sanity_limit() {
    // Recorded: 1.77 ms (75 KB, 21 tracks; 0.5.1, GitHub runner); the limit is 10x 0.5.0's 3.4 ms
    let limit = Duration::from_millis(35);
    let xml = bench_inputs::audio_album_43();
    let mut parser = DDEXParser::new();
    let median = median_time(21, || {
        let parsed = parser.parse(Cursor::new(xml.as_bytes())).unwrap();
        assert_eq!(parsed.flat.releases[0].tracks.len(), 21);
    });
    println!("1 Audio.xml: median {median:?} (limit {limit:?})");
    assert!(
        median < limit,
        "parse took {median:?}, sanity limit {limit:?}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "timing sanity limit: release mode only")]
fn ten_mb_catalog_parses_within_sanity_limit() {
    // Recorded: 214 ms (0.5.1, GitHub runner); the limit is 10x 0.5.0's ~450 ms
    let limit = Duration::from_millis(4500);
    let xml = bench_inputs::large_catalog_43(10 * 1024 * 1024);
    let mut parser = DDEXParser::new();
    let median = median_time(3, || {
        let parsed = parser.parse(Cursor::new(xml.as_bytes())).unwrap();
        assert!(parsed.flat.resources.len() > 4000);
    });
    println!("10 MB catalog: median {median:?} (limit {limit:?})");
    assert!(
        median < limit,
        "parse took {median:?}, sanity limit {limit:?}"
    );
}
