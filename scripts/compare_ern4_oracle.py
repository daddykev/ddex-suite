#!/usr/bin/env python3
"""Compare ddex-parser's flat output for ERN 4.x files with the independent lxml oracle.

Usage (from the repo root, with lxml installed):
    cargo build --release -p ddex-parser --bin ddex-parser
    python scripts/compare_ern4_oracle.py [--parser target/release/ddex-parser] FILE.xml ...

Prints one Markdown table row per file and exits non-zero on any mismatch. Compared per
track, in order: resource reference, ISRC, title, display artist, duration (seconds),
TrackRelease reference/ID, track_number (= item SequenceNumber) and group path.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ern4_oracle import oracle  # noqa: E402


def iso_seconds(value):
    """PT#H#M#S -> whole seconds (fractions truncated, as the parser does)."""
    if value is None:
        return None
    m = re.fullmatch(r"PT(?:(\d+(?:\.\d+)?)H)?(?:(\d+(?:\.\d+)?)M)?(?:(\d+(?:\.\d+)?)S)?", value)
    if not m:
        return None
    h, mi, s = (float(x) if x else 0.0 for x in m.groups())
    return int(h * 3600) + int(mi * 60) + int(s)


def parse_flat(parser, path, tmp):
    out = os.path.join(tmp, "out.json")
    subprocess.run([parser, "parse", "--flatten", "-q", "-o", out, path],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    with open(out, encoding="utf-8") as f:
        return json.load(f)


def expected_rows(rel):
    return [{
        "ref": t["resource_reference"],
        "isrc": t["isrc"],
        "title": t["title"],
        "artist": t["display_artist"],
        "secs": iso_seconds(t["duration"]),
        "tr": (t["track_release_reference"], t["track_release_id"]),
        "track_number": int(t["item_seq"]) if t["item_seq"] else None,
        "path": [(g["type"], int(g["seq"]) if g["seq"] else None) for g in t["group_path"]],
    } for t in rel["tracks"]]


def parsed_rows(rel):
    return [{
        "ref": t["track_id"],
        "isrc": t["isrc"],
        "title": t["title"],
        "artist": t["display_artist"],
        "secs": t["duration"]["secs"] if t["duration"] else None,
        "tr": (t.get("track_release_reference"), t.get("track_release_id")),
        "track_number": t["track_number"],
        "path": [(g["group_type"], g["sequence_number"]) for g in t.get("group_path", [])],
    } for t in rel["tracks"]]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--parser", default="target/release/ddex-parser")
    ap.add_argument("files", nargs="+")
    args = ap.parse_args()

    print("| sample | expected tracks | parsed tracks | ISRCs match | titles match | other fields match |")
    print("|---|---|---|---|---|---|")
    ok = True
    with tempfile.TemporaryDirectory() as tmp:
        for path in args.files:
            exp = oracle(path)
            try:
                flat = parse_flat(args.parser, path, tmp)
            except subprocess.CalledProcessError as e:
                print(f"| {path} | - | PARSE ERROR: {e.stderr.decode().strip()[:80]} | | | |")
                ok = False
                continue
            e_rows = [r for rel in exp["releases"] for r in expected_rows(rel)]
            p_rows = [r for rel in flat["releases"] for r in parsed_rows(rel)]
            isrc = [r["isrc"] for r in e_rows] == [r["isrc"] for r in p_rows]
            title = [r["title"] for r in e_rows] == [r["title"] for r in p_rows]
            other_fields = ("ref", "artist", "secs", "tr", "track_number", "path")
            other = len(e_rows) == len(p_rows) and all(
                all(e[k] == p[k] for k in other_fields) for e, p in zip(e_rows, p_rows))
            if not other:
                for e, p in zip(e_rows, p_rows):
                    bad = [k for k in other_fields if e[k] != p[k]]
                    if bad:
                        print(f"<!-- {path} {e['ref']}: " + ", ".join(
                            f"{k}: expected {e[k]!r} got {p[k]!r}" for k in bad) + " -->")
                        break
            row_ok = len(e_rows) == len(p_rows) and isrc and title and other
            ok &= row_ok
            yn = lambda b: "yes" if b else "**NO**"  # noqa: E731
            print(f"| {os.path.relpath(path)} | {len(e_rows)} | {len(p_rows)} | {yn(isrc)} | {yn(title)} | {yn(other)} |")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
