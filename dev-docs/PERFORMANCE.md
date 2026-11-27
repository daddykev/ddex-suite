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

- ddex-suite 0.6.0 (commit `410315f`, branch `release-0.6.0`), recorded 2026-11-27 by the
  Bench workflow, [run 45985971656](https://github.com/daddykev/ddex-suite/actions/runs/45985971656)
- GitHub-hosted `ubuntu-latest` runner (image ubuntu26 20261116.202.1): Intel Xeon Platinum
  8573C, 4 vCPUs (2 cores × 2 threads), 15 GiB
- Rust 1.100.0, `cargo bench` (bench profile = release: `opt-level = 3`, fat LTO,
  `codegen-units = 1`)
- Criterion defaults (100 samples; 10 for the 10 MB inputs). Values are the median
  estimate. A shared runner is noisier than a dedicated machine; treat differences of a
  few percent as noise.

The 0.5.1 numbers (release notes 0.5.1) were measured on a runner with another CPU (Intel
Xeon 6975P-C), and the 0.5.0 numbers on an Apple M1 Pro; neither is comparable with these.

## ddex-parser

### `DDEXParser::parse` (the default, complete parser)

Official ERN 4.3 samples (`examples/Samples43`, bench `throughput`, group
`official_samples_43`):

| Sample | Size | Time | Throughput |
|---|---:|---:|---:|
| 1 Audio.xml | 75 KB | 2.52 ms | 28.5 MiB/s |
| 2 Video.xml | 13 KB | 0.53 ms | 24.1 MiB/s |
| 3 MixedMedia.xml | 126 KB | 4.21 ms | 28.6 MiB/s |
| 4 SimpleAudioSingle.xml | 330 KB | 11.0 ms | 28.7 MiB/s |
| 5 SimpleVideoSingle.xml | 10 KB | 0.41 ms | 22.1 MiB/s |
| 6 Ringtone.xml | 116 KB | 3.95 ms | 28.1 MiB/s |
| 7 LongformMusicalWorkVideo.xml | 50 KB | 1.55 ms | 30.8 MiB/s |
| 8 DjMix.xml | 19 KB | 0.76 ms | 24.5 MiB/s |
| Variant Classical.xml | 59 KB | 2.01 ms | 27.9 MiB/s |

A ~10 MB ERN 4.3 catalog (`1 Audio.xml` with its sound recordings repeated, about 4,000
resources; group `large_catalog_43`): **323 ms, 31.0 MiB/s**.

The complete parser runs at about 22–31 MiB/s on real ERN 4.3 documents, whatever their
size: the official samples (10–330 KB) take 0.41–11 ms, a typical album message about
2.5 ms.

### Streaming iterators (10 MB catalog, group `streaming_iterators_10MB`)

These iterators yield low-level elements; they do not build the release/track model, so
their numbers are not comparable with `parse`.

| Iterator | Time | Throughput |
|---|---:|---:|
| `WorkingStreamIterator` | 74.9 ms | 134 MiB/s |
| `FastZeroCopyIterator` | 20.1 ms | 498 MiB/s |
| `ParallelStreamingIterator`, 1 thread | 20.8 ms | 482 MiB/s |
| `ParallelStreamingIterator`, 2 threads | 21.1 ms | 473 MiB/s |
| `ParallelStreamingIterator`, 4 threads | 21.0 ms | 477 MiB/s |

The parallel iterator does not scale with the thread count (1, 2 and 4 threads take
20.8–21.1 ms). The runner has 4 vCPUs, so 8 threads were not measured.

## ddex-builder

### `DDEXBuilder::build` (bench `performance`, group `ddex_builder_performance`)

Complete ERN 4.3 requests (`benches/common`: one release, N sound recordings, default
`BuildOptions`, i.e. DB-C14N canonical output):

| Request | Time |
|---|---:|
| 1 track | 0.73 ms |
| 12 tracks | 3.44 ms |
| 100 tracks | 24.4 ms |

Group `xml_generation`: 1 track 0.83 ms, 12 tracks 3.45 ms, 50 tracks 12.5 ms. Bench
`determinism`, group `single_build`: `small` 1.54 ms, `medium` 12.5 ms, `large` 48.5 ms.
The cost is roughly 0.24 ms per track plus about 0.5 ms per message (1–100 tracks:
0.73–24.4 ms).

Other groups:

| Bench | Time |
|---|---:|
| `canonicalization` `canonicalize_small` | 34.3 µs |
| `canonicalization` `canonical_hash` | 0.25 µs |
| `performance` `memory_allocation/request_clone_12_tracks` | 4.8 µs |

(`memory_allocation/builder_creation` is not reported: `DDEXBuilder` is a unit struct since
0.6.0, so the bench measures nothing; criterion gives about 0 ns.)

### Security helpers (bench `security`)

| Operation | Baseline | With the helper |
|---|---:|---:|
| 4 paths: `Path::new(..).components()` vs `PathValidator::validate` | 0.11 µs | 23.8 µs |

Path validation costs about 6 µs per path, which is negligible next to a build.

## Sanity limits (`--features perf-tests`, release)

| Test | Recorded | Limit |
|---|---:|---:|
| ddex-parser `perf::typical_album_parses_within_sanity_limit` (1 Audio.xml) | 2.66 ms | 35 ms |
| ddex-parser `perf::ten_mb_catalog_parses_within_sanity_limit` | 325 ms | 4.5 s |
| ddex-builder `perf::single_track_builds_within_sanity_limit` | 1.27 ms | 28 ms |
| ddex-builder `perf::twelve_track_album_builds_within_sanity_limit` | 3.51 ms | 115 ms |
| ddex-builder `perf::hundred_track_compilation_builds_within_sanity_limit` | 24.2 ms | 810 ms |

In debug builds these tests are ignored (debug is 50–100× slower).

## Changes

- 0.6.0: recorded on a GitHub-hosted runner with a different CPU (Xeon Platinum 8573C; 0.5.1
  ran on a Xeon 6975P-C), so the change from 0.5.1 is not a like-for-like comparison. The
  `DeterminismVerifier`, `parallel_processor`, entity-classification and
  error-sanitization groups were removed with that code.

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
