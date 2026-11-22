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

- **Machine**: Apple M1 Pro (8 cores), 16 GB, macOS 15.8.3
- **Toolchain**: Rust 1.89.0, `cargo bench` (release profile: `opt-level = 3`, fat LTO,
  `codegen-units = 1`)
- **Measurement**: criterion defaults (100 samples; 10 for the 10 MB inputs), median
  estimate
- **Date**: 2026-11-22, ddex-suite 0.5.0

```bash
cargo bench -p ddex-parser --bench throughput --bench fast_streaming_benchmark
cargo bench -p ddex-builder --bench performance --bench security --bench canonicalization --bench determinism
```

The numbers are for the Rust libraries. The Node.js and Python bindings run the same code,
plus the cost of converting the result to JavaScript or Python objects, which was not
measured separately.

## Parsing

`DDEXParser::parse`, the complete parser, on the official ERN 4.3 samples:

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
resources) parses in **441 ms (22.7 MiB/s)**.

The parser runs at about 20–24 MiB/s on real ERN 4.3 documents whatever their size: a
typical album message takes a few milliseconds.

## Building

`DDEXBuilder::build` with complete ERN 4.3 requests (one release, N sound recordings,
default options, DB-C14N canonical output):

| Request | Time |
|---|---:|
| 1 track | 2.73 ms |
| 12 tracks | 11.4 ms |
| 50 tracks | 41.7 ms |
| 100 tracks | 80.4 ms |

That is roughly 0.8 ms per track plus about 2 ms per message.

## Streaming iterators

The Rust streaming iterators yield low-level elements and do not build the release and track
model, so they are not comparable with `parse`. On the 10 MB catalog:

| Iterator | Time | Throughput |
|---|---:|---:|
| `WorkingStreamIterator` | 59.8 ms | 167 MiB/s |
| `FastZeroCopyIterator` | 11.7 ms | 854 MiB/s |
| `ParallelStreamingIterator`, 1 thread | 11.6 ms | 862 MiB/s |
| `ParallelStreamingIterator`, 8 threads | 11.6 ms | 866 MiB/s |

The parallel iterator does not scale with the thread count (1, 2, 4 and 8 threads all take
11.6 ms); making it scale is planned for 0.6.0.

## Regression limits

CI does not time anything. Behind the `perf-tests` feature (release builds only) a few tests
fail if an operation becomes about 10 times slower than the numbers above, for example the
typical album parse (limit 35 ms) or the 100-track build (limit 810 ms). Debug builds are
50–100 times slower than release builds.
