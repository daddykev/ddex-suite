# Performance numbers

Recorded numbers, not targets. They come from the criterion benches; the only timing
assertions left are the sanity limits behind the `perf-tests` feature (about 10× these
numbers, release mode only), which catch catastrophic regressions:

```bash
cargo bench -p ddex-parser --bench throughput --bench fast_streaming_benchmark
cargo bench -p ddex-builder --bench performance --bench security --bench canonicalization --bench determinism
cargo test --release --features ddex-parser/perf-tests,ddex-builder/perf-tests perf::
```

Re-record this file when a change is expected to move the numbers. Older figures
(`PERFORMANCE_ACHIEVEMENT.md`, the "Performance" sections of README / CLAUDE.md) were not
produced this way and are superseded by this file.

## Machine and run

- Date: 2026-11-22, ddex-suite 0.5.0 (release-0.5.0 branch)
- Apple M1 Pro (8 cores), 16 GB, macOS 15.8.3
- Rust 1.89.0, `cargo bench` (bench profile = release: `opt-level = 3`, fat LTO,
  `codegen-units = 1`)
- Criterion defaults (100 samples; 10 for the 10 MB inputs). Values are the median
  estimate.

## ddex-parser

### `DDEXParser::parse` (the default, complete parser)

Official ERN 4.3 samples (`examples/Samples43`, bench `throughput`, group
`official_samples_43`):

| Sample | Size | Time | Throughput |
|---|---:|---:|---:|
| 1 Audio.xml | 75 KB | 3.39 ms | 21.2 MiB/s |
| 2 Video.xml | 13 KB | 0.64 ms | 19.7 MiB/s |
| 3 MixedMedia.xml | 126 KB | 5.40 ms | 22.3 MiB/s |
| 4 SimpleAudioSingle.xml | 330 KB | 14.3 ms | 22.0 MiB/s |
| 5 SimpleVideoSingle.xml | 10 KB | 0.50 ms | 18.2 MiB/s |
| 6 Ringtone.xml | 116 KB | 4.91 ms | 22.6 MiB/s |
| 7 LongformMusicalWorkVideo.xml | 50 KB | 1.97 ms | 24.2 MiB/s |
| 8 DjMix.xml | 19 KB | 0.94 ms | 19.7 MiB/s |
| Variant Classical.xml | 59 KB | 2.68 ms | 20.9 MiB/s |

A ~10 MB ERN 4.3 catalog (`1 Audio.xml` with its sound recordings repeated, about 4,000
resources; group `large_catalog_43`): **441 ms, 22.7 MiB/s**.

The complete parser runs at about 20–24 MiB/s on real ERN 4.3 documents, whatever their
size; a typical album message parses in a few milliseconds.

### Streaming iterators (10 MB catalog, group `streaming_iterators_10MB`)

These iterators yield low-level elements; they do not build the release/track model, so
their numbers are not comparable with `parse`.

| Iterator | Time | Throughput |
|---|---:|---:|
| `WorkingStreamIterator` | 59.8 ms | 167 MiB/s |
| `FastZeroCopyIterator` | 11.7 ms | 854 MiB/s |
| `ParallelStreamingIterator`, 1 thread | 11.6 ms | 862 MiB/s |
| `ParallelStreamingIterator`, 2 threads | 11.6 ms | 861 MiB/s |
| `ParallelStreamingIterator`, 4 threads | 11.6 ms | 863 MiB/s |
| `ParallelStreamingIterator`, 8 threads | 11.6 ms | 866 MiB/s |

The parallel iterator does not scale with the thread count (the earlier "6.25× on 8
cores" claim is not reproduced).

### `FastStreamingParser` (bench `fast_streaming_benchmark`)

About 3 GiB/s on its generated inputs (e.g. "10MB" group: 0.81 ms; "100MB": 4.1 ms).
Caveats: this is the fast path behind `SecurityConfig::relaxed()`, which returns no
releases or resources (KNOWN_ISSUES.md), so it measures scanning, not extraction; the
group labels are nominal (the "50MB" inputs are about 11 MB); and the parser's
self-reported peak memory for the `memory_efficiency` input is 50 MB (its `max_memory`
setting is not applied).

## ddex-builder

### `DDEXBuilder::build` (bench `performance`, group `ddex_builder_performance`)

Complete ERN 4.3 requests (`benches/common`: one release, N sound recordings, default
`BuildOptions`, i.e. DB-C14N canonical output):

| Request | Time |
|---|---:|
| 1 track | 2.73 ms |
| 12 tracks | 11.4 ms |
| 50 tracks (`xml_generation`) | 41.7 ms |
| 100 tracks | 80.4 ms |

Bench `determinism`, group `single_build`: 5 tracks 5.94 ms, 50 tracks 41.6 ms,
200 tracks 159 ms. (Its `determinism_verification` groups, which build each request 2–5
times, were not recorded.) The cost is roughly 0.8 ms per track plus about 2 ms per
message; `DDEXBuilder::new()` alone is 73 µs.

Other groups:

| Bench | Time |
|---|---:|
| `canonicalization` `canonicalize_small` | 138 µs |
| `canonicalization` `canonical_hash` | 0.79 µs |
| `performance` `parallel_processor` 1 / 12 / 50 tracks | 0.43 µs / 33 µs / 56 µs |

`ParallelProcessor::process_build_parallel` does not write XML (it returns in under a
microsecond for one track), so its numbers are not comparable with `build`.

### Security helpers (bench `security`)

| Operation | Baseline | With the helper |
|---|---:|---:|
| 4 paths: `Path::new(..).components()` vs `PathValidator::validate` | 99 ns | 175 µs |
| 3 entities: string lengths vs `EntityClassifier::validate_entity_chain` | 1.6 ns | 486 ns |
| 3 I/O errors: `format!` vs `ErrorSanitizer::sanitize_io_error` | 673 ns | 8.1 µs |

The removed test asserted that each helper adds less than 5 % over its baseline; they add
far more than that relative to these trivial baselines (path validation is about 44 µs per
path), which is negligible next to a build.

## Sanity limits (`--features perf-tests`, release)

| Test | Recorded | Limit |
|---|---:|---:|
| ddex-parser `perf::typical_album_parses_within_sanity_limit` (1 Audio.xml) | ~3.4 ms | 35 ms |
| ddex-parser `perf::ten_mb_catalog_parses_within_sanity_limit` | ~450 ms | 4.5 s |
| ddex-builder `perf::single_track_builds_within_sanity_limit` | ~2.8 ms | 28 ms |
| ddex-builder `perf::twelve_track_album_builds_within_sanity_limit` | ~11.5 ms | 115 ms |
| ddex-builder `perf::hundred_track_compilation_builds_within_sanity_limit` | ~81 ms | 810 ms |

In debug builds these tests are ignored (debug is 50–100× slower).

## Changes in 0.5.0

The wall-clock pass/fail tests (`perf-tests`) became these benches. The removed tests
asserted targets that the machine above missed intermittently, and two sets measured
nothing useful: the ddex-builder perf tests and benches used an incomplete ERN 4.3
request, so they timed the validation error (the benches now build for real and unwrap
the result), and their "memory" figure was `process::id() * 1024`; the ddex-parser
throughput tests ran the `SecurityConfig::relaxed()` fast path, which returns no
releases.
