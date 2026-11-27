---
sidebar_position: 10
title: Performance
description: Measured parse and build times from the criterion benches, with the method and the machine.
---

# Performance

These are recorded numbers, not targets. They come from the
[criterion](https://github.com/bheisler/criterion.rs) benches in the repository, and the
full results are in
[dev-docs/PERFORMANCE.md](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/PERFORMANCE.md).

## Method and machine

- **Machine**: GitHub-hosted `ubuntu-latest` runner, Intel Xeon 6975P-C, 4 vCPUs, 15 GiB
  (the repository's Bench workflow, `.github/workflows/bench.yml`)
- **Toolchain**: Rust 1.100.0, `cargo bench` (release profile: `opt-level = 3`, fat LTO,
  `codegen-units = 1`)
- **Measurement**: criterion defaults (100 samples; 10 for the 10 MB inputs), median
  estimate. A shared runner is noisier than a dedicated machine.
- **Date**: 2026-11-26, ddex-suite 0.5.1

```bash
cargo bench -p ddex-parser --bench throughput
cargo bench -p ddex-builder --bench performance --bench security --bench canonicalization --bench determinism
```

The numbers are for the Rust libraries. The Node.js and Python bindings run the same code,
plus the cost of converting the result to JavaScript or Python objects, which was not
measured separately.

## Parsing

`DDEXParser::parse`, the complete parser, on the official ERN 4.3 samples:

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
resources) parses in **211 ms (47.3 MiB/s)**.

The parser runs at about 33–47 MiB/s on real ERN 4.3 documents whatever their size: a
typical album message takes under 2 milliseconds.

## Building

`DDEXBuilder::build` with complete ERN 4.3 requests (one release, N sound recordings,
default options, DB-C14N canonical output):

| Request | Time |
|---|---:|
| 1 track | 1.72 ms |
| 12 tracks | 8.29 ms |
| 50 tracks | 29.6 ms |
| 100 tracks | 57.8 ms |

That is roughly 0.56 ms per track plus 1–1.5 ms per message.

## Streaming iterators

The Rust streaming iterators yield low-level elements and do not build the release and track
model, so they are not comparable with `parse`. On the 10 MB catalog:

| Iterator | Time | Throughput |
|---|---:|---:|
| `WorkingStreamIterator` | 55.5 ms | 180 MiB/s |
| `FastZeroCopyIterator` | 15.3 ms | 655 MiB/s |
| `ParallelStreamingIterator`, 1 thread | 16.1 ms | 621 MiB/s |
| `ParallelStreamingIterator`, 4 threads | 16.1 ms | 622 MiB/s |

The parallel iterator does not scale with the thread count (1, 2 and 4 threads all take
16.1 ms; the runner has 4 vCPUs).

## Regression limits

CI does not time anything. Behind the `perf-tests` feature (release builds only) a few tests
fail if an operation becomes about 10 times slower than the 0.5.0 numbers, for example the
typical album parse (limit 35 ms) or the 100-track build (limit 810 ms). Debug builds are
50–100 times slower than release builds.
