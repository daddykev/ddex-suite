#!/usr/bin/env python3
"""Which 0.5.0 fields ddex-parser populates for ERN 3.8.x files, checked against lxml.

Usage (from the repo root, with lxml installed):
    cargo build --release -p ddex-parser --bin ddex-parser
    python scripts/ern38_population.py [--parser target/release/ddex-parser] FILE.xml ...

For each file and field, a cell "populated/present": in how many releases (or tracks)
lxml finds the field in the XML, and in how many of those the parser's flat output has a
value. A field the parser fills where the XML has none, or misses where it has one, is a
mismatch (exit 1). "-" means the field is absent from the file.
"""
import argparse
import json
import os
import subprocess
import sys
import tempfile

from lxml import etree

DETAILS = ("SoundRecordingDetailsByTerritory", "VideoDetailsByTerritory",
           "ImageDetailsByTerritory", "TextDetailsByTerritory", "SheetMusicDetailsByTerritory")


def ln(tag):
    return etree.QName(tag).localname if isinstance(tag, str) else None


def kids(el, *names):
    return [c for c in el if ln(c.tag) in names]


def has(el, *path):
    """Whether a child path exists (each step: a set of names)."""
    level = [el]
    for names in path:
        level = [c for e in level for c in kids(e, *names)]
    return bool(level)


RELEASE_FIELDS = {
    "ReleaseId ISRC": (lambda r: has(r, ["ReleaseId"], ["ISRC"]), lambda p: p["identifiers"].get("isrc")),
    "ReleaseId ProprietaryId": (lambda r: has(r, ["ReleaseId"], ["ProprietaryId"]),
                                lambda p: p["identifiers"]["proprietary"]),
    "label": (lambda r: has(r, ["ReleaseDetailsByTerritory"], ["LabelName"]), lambda p: p["label_name"]),
    "genre": (lambda r: has(r, ["ReleaseDetailsByTerritory"], ["Genre"]), lambda p: p["genre"]),
    "parental warning": (lambda r: has(r, ["ReleaseDetailsByTerritory"], ["ParentalWarningType"]),
                         lambda p: p["parental_warning"]),
    "P line": (lambda r: has(r, ["PLine"]) or has(r, ["ReleaseDetailsByTerritory"], ["PLine"]),
               lambda p: p["p_line"]),
    "C line": (lambda r: has(r, ["CLine"]) or has(r, ["ReleaseDetailsByTerritory"], ["CLine"]),
               lambda p: p["c_line"]),
    "release dates": (lambda r: has(r, ["GlobalReleaseDate"]) or has(r, ["ReleaseDetailsByTerritory"], ["ReleaseDate"]),
                      lambda p: p["release_dates"]),
    "original dates": (lambda r: has(r, ["GlobalOriginalReleaseDate"])
                       or has(r, ["ReleaseDetailsByTerritory"], ["OriginalReleaseDate"]),
                       lambda p: p["original_release_dates"]),
    "ResourceGroup (track numbers)": (lambda r: has(r, ["ReleaseDetailsByTerritory"], ["ResourceGroup"]),
                                      lambda p: any(t.get("track_number") is not None for t in p["tracks"])),
}

TRACK_FIELDS = {
    "track artists": (lambda s: has(s, DETAILS, ["DisplayArtist", "DisplayArtistName"]), lambda t: t["display_artist"]),
    "track label": (lambda s: has(s, DETAILS, ["LabelName"]), lambda t: t["label_name"]),
    "track genre": (lambda s: has(s, DETAILS, ["Genre"]), lambda t: t["genre"]),
    "track parental warning": (lambda s: has(s, DETAILS, ["ParentalWarningType"]), lambda t: t["parental_warning"]),
    "track P line": (lambda s: has(s, DETAILS, ["PLine"]), lambda t: t["p_line"]),
}

COLUMNS = list(RELEASE_FIELDS) + list(TRACK_FIELDS) + ["MessageControlType", "deal EndDate"]


def parse_flat(parser, path, tmp):
    out = os.path.join(tmp, "out.json")
    subprocess.run([parser, "parse", "--flatten", "-q", "-o", out, path],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    with open(out, encoding="utf-8") as f:
        return json.load(f)


def row(path, flat):
    root = etree.parse(path).getroot()
    releases = [r for l in kids(root, "ReleaseList") for r in kids(l, "Release")]
    resources = {}
    for l in kids(root, "ResourceList"):
        for res in l:
            refs = kids(res, "ResourceReference")
            if refs:
                resources[" ".join((refs[0].text or "").split())] = res
    cells, ok = {}, True

    def cell(name, pairs):
        nonlocal ok
        present = sum(1 for x, _ in pairs if x)
        populated = sum(1 for x, y in pairs if x and y)
        invented = sum(1 for x, y in pairs if y and not x)
        ok &= populated == present and invented == 0
        cells[name] = "-" if present == 0 and invented == 0 else (
            f"{populated}/{present}" + (f" (+{invented} not in XML)" if invented else ""))

    assert len(releases) == len(flat["releases"]), path
    for name, (in_xml, in_flat) in RELEASE_FIELDS.items():
        cell(name, [(in_xml(r), bool(in_flat(p))) for r, p in zip(releases, flat["releases"])])
    for name, (in_xml, in_flat) in TRACK_FIELDS.items():
        cell(name, [(in_xml(resources[t["track_id"]]), bool(in_flat(t)))
                    for p in flat["releases"] for t in p["tracks"]])
    header = kids(root, "MessageHeader")[0]
    cell("MessageControlType", [(has(header, ["MessageControlType"]), bool(flat.get("message_control_type")))])
    deals = [d for l in kids(root, "DealList") for rd in kids(l, "ReleaseDeal") for d in kids(rd, "Deal")]
    cell("deal EndDate", [(has(d, ["DealTerms"], ["ValidityPeriod"], ["EndDate"]), bool(f["validity"]["end"]))
                          for d, f in zip(deals, flat["deals"])])
    return cells, ok


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--parser", default="target/release/ddex-parser")
    ap.add_argument("files", nargs="+")
    args = ap.parse_args()
    print("| file | " + " | ".join(COLUMNS) + " |")
    print("|---|" + "---|" * len(COLUMNS))
    all_ok = True
    with tempfile.TemporaryDirectory() as tmp:
        for path in args.files:
            cells, ok = row(path, parse_flat(args.parser, path, tmp))
            all_ok &= ok
            print(f"| {os.path.basename(path)} | " + " | ".join(cells[c] for c in COLUMNS) + " |")
    sys.exit(0 if all_ok else 1)


if __name__ == "__main__":
    main()
