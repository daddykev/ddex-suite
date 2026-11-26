#!/usr/bin/env python3
"""Summarize criterion results as a Markdown table: one row per benchmark, with the median
estimate (the figure dev-docs/PERFORMANCE.md records) and, where the bench declares a byte
throughput, MiB/s from that median.

Usage: python3 scripts/criterion_summary.py target/criterion
Reads <dir>/**/new/estimates.json and the matching benchmark.json; writes nothing else.
"""

import json
import sys
from pathlib import Path


def fmt_time(ns):
    for unit, scale in (("s", 1e9), ("ms", 1e6), ("µs", 1e3)):
        if ns >= scale:
            return f"{ns / scale:.3g} {unit}"
    return f"{ns:.3g} ns"


def main(root):
    rows = []
    for estimates in sorted(Path(root).glob("**/new/estimates.json")):
        bench = json.loads((estimates.parent / "benchmark.json").read_text())
        median = json.loads(estimates.read_text())["median"]["point_estimate"]
        throughput = bench.get("throughput") or {}
        mib_s = ""
        if "Bytes" in throughput:
            mib_s = f"{throughput['Bytes'] / (median / 1e9) / 2**20:.1f} MiB/s"
        rows.append((bench["full_id"], fmt_time(median), mib_s))
    if not rows:
        sys.exit(f"no criterion results under {root}")
    print("| Benchmark | Median | Throughput |")
    print("|---|---:|---:|")
    for row in rows:
        print("| `{}` | {} | {} |".format(*row))


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "target/criterion")
