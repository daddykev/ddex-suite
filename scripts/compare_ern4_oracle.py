#!/usr/bin/env python3
"""Compare ddex-parser's flat output for ERN 4.x files with the independent lxml oracle.

Usage (from the repo root, with lxml installed):
    cargo build --release -p ddex-parser --bin ddex-parser
    python scripts/compare_ern4_oracle.py [--parser target/release/ddex-parser] FILE.xml ...

Prints one Markdown table row per file and exits non-zero on any mismatch. Compared per
track, in order: resource reference, ISRC, title, display artist, duration (seconds),
TrackRelease reference/ID, track_number (= item SequenceNumber) and group path.

A second table compares the 0.4.6 fields (ern4_oracle.details) per sample: one yes/no per
field, plus how many of the fields are populated in the sample.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ern4_oracle import details, oracle  # noqa: E402


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


DETAIL_COLUMNS = [
    "control type", "genre", "genres", "parental warning", "parental warnings", "label",
    "P line", "C line", "release date", "release dates", "original date",
    "original dates", "track warning", "track P/C line", "TrackRelease genre", "deal dates",
]


def parsed_line(c):
    return {"text": c["text"], "year": c["year"]} if c else None


def scope_territory(item):
    codes = item.get("territory_codes") or []
    return codes[0] if len(codes) == 1 else None


def detail_checks(exp, flat):
    """Per DETAIL_COLUMNS, whether the parser matches the oracle."""
    checks = {k: True for k in DETAIL_COLUMNS}
    checks["control type"] = exp["message_control_type"] == flat.get("message_control_type")
    if len(exp["releases"]) != len(flat["releases"]):
        return {k: False for k in DETAIL_COLUMNS}
    for e, p in zip(exp["releases"], flat["releases"]):
        checks["genre"] &= (e["genre"], e["sub_genre"]) == (p["genre"], p["sub_genre"])
        checks["genres"] &= e["genres"] == [
            {"genre": g["genre"], "sub_genre": g["sub_genre"], "territory": scope_territory(g),
             "is_default": g.get("is_default")} for g in p["genres"]]
        checks["parental warning"] &= e["parental_warning"] == p["parental_warning"]
        checks["parental warnings"] &= e["parental_warnings"] == p["parental_warnings"]
        checks["label"] &= e["label_name"] == p["label_name"]
        checks["P line"] &= e["p_line"] == parsed_line(p["p_line"])
        checks["C line"] &= e["c_line"] == parsed_line(p["c_line"])
        checks["release date"] &= e["release_date"] == p["release_date"]
        checks["release dates"] &= e["release_dates"] == [
            {"date": d["date"], "territory": scope_territory(d)} for d in p["release_dates"]]
        checks["original date"] &= e["original_release_date"] == p["original_release_date"]
        checks["original dates"] &= e["original_release_dates"] == [
            {"date": d["date"], "territory": scope_territory(d)} for d in p["original_release_dates"]]
        if len(e["tracks"]) != len(p["tracks"]):
            for k in ("track warning", "track P/C line", "TrackRelease genre"):
                checks[k] = False
            continue
        for et, pt in zip(e["tracks"], p["tracks"]):
            checks["track warning"] &= et["parental_warning"] == pt["parental_warning"]
            checks["track P/C line"] &= (et["p_line"], et["c_line"]) == (
                parsed_line(pt["p_line"]), parsed_line(pt["c_line"]))
            checks["TrackRelease genre"] &= (et["track_release_genre"], et["track_release_sub_genre"]) == (
                pt["track_release_genre"], pt["track_release_sub_genre"])
    checks["deal dates"] = [(d["start"], d["end"]) for d in exp["deals"]] == [
        (d["validity"]["start"], d["validity"]["end"]) for d in flat["deals"]]
    return checks


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
    # 0.4.6 fields
    print()
    print("| sample | " + " | ".join(DETAIL_COLUMNS) + " | populated |")
    print("|---|" + "---|" * (len(DETAIL_COLUMNS) + 1))
    with tempfile.TemporaryDirectory() as tmp:
        for path in args.files:
            try:
                flat = parse_flat(args.parser, path, tmp)
            except subprocess.CalledProcessError:
                ok = False
                continue
            exp = details(path)
            checks = detail_checks(exp, flat)
            ok &= all(checks.values())
            rels = exp["releases"]
            populated = sorted({k for r in rels for k, v in r.items() if v and k != "tracks"}
                               | {"tracks." + k for r in rels for t in r["tracks"] for k, v in t.items() if v and k != "ref"}
                               | ({"control type"} if exp["message_control_type"] else set())
                               | ({"deal end"} if any(d["end"] for d in exp["deals"]) else set()))
            yn = lambda b: "yes" if b else "**NO**"  # noqa: E731
            print(f"| {os.path.relpath(path)} | " + " | ".join(yn(checks[k]) for k in DETAIL_COLUMNS)
                  + f" | {len(populated)} |")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
