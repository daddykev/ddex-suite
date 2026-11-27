# Performance numbers

Recorded numbers, not targets. They come from the criterion benches; the only timing
assertions left are the sanity limits behind the `perf-tests` feature (about 10× the 0.5.0
numbers, release mode only), which catch catastrophic regressions:

```bash
cargo bench -p ddex-parser --bench throughput
cargo bench -p ddex-builder --bench performance --bench security --bench canonicalization --bench determinism
cargo test --release --features ddex-parser/perf-tests,ddex-builder/perf-tests perf::
```

The **Bench** workflow (`.github/workflows/bench.yml`, run by hand) runs exactly these on a
GitHub-hosted runner and prints the medians (`scripts/criterion_summary.py`) and the
machine. Re-record this file from its output when a change is expected to move the
numbers. Older figures (`PERFORMANCE_ACHIEVEMENT.md`, `benchmark-suite/`) were not
produced this way and are superseded by this file.

## Machine and run

- ddex-suite 0.5.1 code (commit `460eed7`, branch `bench/0.5.1`; the version string still
  read 0.5.0, the bump came later), recorded 2026-11-26 by the Bench workflow,
  [run 45797376953](https://github.com/daddykev/ddex-suite/actions/runs/45797376953)
- GitHub-hosted `ubuntu-latest` runner (image ubuntu26 20261116.88.1): Intel Xeon 6975P-C,
  4 vCPUs (2 cores × 2 threads), 15 GiB
- Rust 1.100.0, `cargo bench` (bench profile = release: `opt-level = 3`, fat LTO,
  `codegen-units = 1`)
- Criterion defaults (100 samples; 10 for the 10 MB inputs). Values are the median
  estimate. A shared runner is noisier than a dedicated machine; treat differences of a
  few percent as noise.

The 0.5.0 numbers in the 0.5.0 release notes were measured on another machine (an Apple
M1 Pro) and are not comparable with these.

## ddex-parser

### `DDEXParser::parse` (the default, complete parser)

Official ERN 4.3 samples (`examples/Samples43`, bench `throughput`, group
`official_samples_43`):

| Sample | Size | Time | Throughput |
|---|---:|---:|---:|
| 1 Audio.xml | 75 KB | 1.70 ms | 42.3 MiB/s |
| 2 Video.xml | 13 KB | 0.35 ms | 36.3 MiB/s |
| 3 MixedMedia.xml | 126 KB | 2.84 ms | 42.4 MiB/s |
| 4 SimpleAudioSingle.xml | 330 KB | 7.43 ms | 42.4 MiB/s |
| 5 SimpleVideoSingle.xml | 10 KB | 0.27 ms | 33.4 MiB/s |
| 6 Ringtone.xml | 116 KB | 2.68 ms | 41.4 MiB/s |
| 7 LongformMusicalWorkVideo.xml | 50 KB | 1.02 ms | 46.6 MiB/s |
| 8 DjMix.xml | 19 KB | 0.51 ms | 36.4 MiB/s |
| Variant Classical.xml | 59 KB | 1.36 ms | 41.2 MiB/s |

A ~10 MB ERN 4.3 catalog (`1 Audio.xml` with its sound recordings repeated, about 4,000
resources; group `large_catalog_43`): **211 ms, 47.3 MiB/s**.

The complete parser runs at about 33–47 MiB/s on real ERN 4.3 documents, whatever their
size; a typical album message parses in under 2 milliseconds.

### Streaming iterators (10 MB catalog, group `streaming_iterators_10MB`)

These iterators yield low-level elements; they do not build the release/track model, so
their numbers are not comparable with `parse`.

| Iterator | Time | Throughput |
|---|---:|---:|
| `WorkingStreamIterator` | 55.5 ms | 180 MiB/s |
| `FastZeroCopyIterator` | 15.3 ms | 655 MiB/s |
| `ParallelStreamingIterator`, 1 thread | 16.1 ms | 621 MiB/s |
| `ParallelStreamingIterator`, 2 threads | 16.1 ms | 620 MiB/s |
| `ParallelStreamingIterator`, 4 threads | 16.1 ms | 622 MiB/s |

The parallel iterator does not scale with the thread count (it runs on one thread; a
scaling one is planned for 0.6.0). The runner has 4 vCPUs, so 8 threads were not
measured.

## ddex-builder

### `DDEXBuilder::build` (bench `performance`, group `ddex_builder_performance`)

Complete ERN 4.3 requests (`benches/common`: one release, N sound recordings, default
`BuildOptions`, i.e. DB-C14N canonical output):

| Request | Time |
|---|---:|
| 1 track | 1.72 ms |
| 12 tracks | 8.29 ms |
| 50 tracks (`xml_generation`) | 29.6 ms |
| 100 tracks | 57.8 ms |

Bench `determinism`, group `single_build`: 5 tracks 3.62 ms, 50 tracks 29.5 ms,
200 tracks 115 ms. The cost is roughly 0.56 ms per track plus 1–1.5 ms per message;
`DDEXBuilder::new()` alone is 63 µs. (The bench's `DeterminismVerifier` groups were removed
with the verifier in 0.6.0.)

Other groups:

| Bench | Time |
|---|---:|
| `canonicalization` `canonicalize_small` | 84.3 µs |
| `canonicalization` `canonical_hash` | 0.19 µs |

(The `performance` `parallel_processor` group and the `security` bench's entity-classification
and error-sanitization groups were removed with that code in 0.6.0.)

### Security helpers (bench `security`)

| Operation | Baseline | With the helper |
|---|---:|---:|
| 4 paths: `Path::new(..).components()` vs `PathValidator::validate` | 77 ns | 17 µs |

Path validation adds far more than 5 % over this trivial baseline (about 4 µs per path),
which is negligible next to a build.

## Sanity limits (`--features perf-tests`, release)

| Test | Recorded | Limit |
|---|---:|---:|
| ddex-parser `perf::typical_album_parses_within_sanity_limit` (1 Audio.xml) | 1.77 ms | 35 ms |
| ddex-parser `perf::ten_mb_catalog_parses_within_sanity_limit` | 214 ms | 4.5 s |
| ddex-builder `perf::single_track_builds_within_sanity_limit` | 2.00 ms | 28 ms |
| ddex-builder `perf::twelve_track_album_builds_within_sanity_limit` | 8.38 ms | 115 ms |
| ddex-builder `perf::hundred_track_compilation_builds_within_sanity_limit` | 58.0 ms | 810 ms |

In debug builds these tests are ignored (debug is 50–100× slower).

## Changes

- 0.5.1: recorded on a GitHub-hosted runner by the Bench workflow (0.5.0 was recorded on an
  Apple M1 Pro). The 0.5.1 code has the quick-xml 0.41 reader and an extra pass that checks
  every input for references to undefined entities.
- 0.5.0: the wall-clock pass/fail tests (`perf-tests`) became these benches. The removed
  tests asserted targets that the machine missed intermittently, and two sets measured
  nothing useful: the ddex-builder perf tests and benches used an incomplete ERN 4.3
  request, so they timed the validation error (the benches now build for real and unwrap
  the result), and their "memory" figure was `process::id() * 1024`; the ddex-parser
  throughput tests ran the `SecurityConfig::relaxed()` fast path, which returns no
  releases.
