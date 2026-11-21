#!/usr/bin/env python3
"""Independent oracle for ERN 4.x track extraction.

Usage:
    python -m venv .venv && .venv/bin/pip install lxml
    .venv/bin/python scripts/ern4_oracle.py examples/Samples43/*.xml > expected.json

    # Compare with the parser (build it first: cargo build --release -p ddex-parser):
    .venv/bin/python scripts/compare_ern4_oracle.py examples/Samples42/*.xml examples/Samples43/*.xml

Reads ERN 4.2/4.3 files with lxml (NOT the ddex-parser) and prints JSON: per main
Release, the track-eligible ResourceGroupContentItems (SoundRecording/Video) in document
order with ISRC, title (DisplayTitleText, else DisplayTitle/TitleText, IsDefault first),
display artist (DisplayArtistName, else PartyList names), Duration, the enclosing group
path, the item SequenceNumber and the linked TrackRelease.
"""
import json
import sys

from lxml import etree


def ln(tag):
    return etree.QName(tag).localname if isinstance(tag, str) else None


def kids(el, name):
    return [c for c in el if ln(c.tag) == name]


def kid(el, name):
    found = kids(el, name)
    return found[0] if found else None


def text(el):
    return el.text.strip() if el is not None and el.text and el.text.strip() else None


def pick_default(els):
    """IsDefault="true" element if any, else the first."""
    for e in els:
        if e.get("IsDefault") == "true":
            return e
    return els[0] if els else None


def title_of(el):
    t = text(kid(el, "DisplayTitleText"))
    if t:
        return t
    dt = pick_default(kids(el, "DisplayTitle"))
    if dt is not None:
        return text(kid(dt, "TitleText"))
    return None


def isrc_of(res):
    # 4.2: Resource/ResourceId/ISRC ; 4.3: Resource/*Edition/ResourceId/ISRC
    for rid in res.xpath("./*[local-name()='ResourceId'] | ./*[local-name()='SoundRecordingEdition' or local-name()='VideoEdition']/*[local-name()='ResourceId']"):
        v = text(kid(rid, "ISRC"))
        if v:
            return v
    return None


def oracle(path):
    root = etree.parse(path).getroot()
    parties = {}
    for p in root.xpath("./*[local-name()='PartyList']/*[local-name()='Party']"):
        ref = text(kid(p, "PartyReference"))
        names = kids(p, "PartyName")
        full = text(kid(names[0], "FullName")) if names else None
        if ref:
            parties[ref] = full

    resources = {}
    for r in root.xpath("./*[local-name()='ResourceList']/*"):
        ref = text(kid(r, "ResourceReference"))
        if ref:
            resources[ref] = r

    track_releases = {}
    for tr in root.xpath("./*[local-name()='ReleaseList']/*[local-name()='TrackRelease']"):
        rref = text(kid(tr, "ReleaseResourceReference"))
        rid_el = kid(tr, "ReleaseId")
        rid = None
        if rid_el is not None:
            for c in rid_el:
                if text(c):
                    rid = text(c)
                    break
        track_releases.setdefault(rref, (text(kid(tr, "ReleaseReference")), rid))

    out = []
    for rel in root.xpath("./*[local-name()='ReleaseList']/*[local-name()='Release']"):
        tracks = []

        def walk(group, path):
            for c in group:
                name = ln(c.tag)
                if name == "ResourceGroup":
                    walk(c, path + [{"type": c.get("ResourceGroupType"),
                                     "seq": text(kid(c, "SequenceNumber"))}])
                elif name == "ResourceGroupContentItem":
                    ref = text(kid(c, "ReleaseResourceReference"))
                    res = resources.get(ref)
                    if res is None or ln(res.tag) not in ("SoundRecording", "Video"):
                        continue
                    dan = pick_default(kids(res, "DisplayArtistName"))
                    artist = text(dan)
                    if artist is None:
                        names = [parties.get(text(kid(da, "ArtistPartyReference")))
                                 for da in kids(res, "DisplayArtist")]
                        names = [n for n in names if n]
                        artist = ", ".join(names) or None
                    tr = track_releases.get(ref, (None, None))
                    tracks.append({
                        "group_path": path,
                        "item_seq": text(kid(c, "SequenceNumber")),
                        "resource_reference": ref,
                        "resource_type": ln(res.tag),
                        "isrc": isrc_of(res),
                        "title": title_of(res),
                        "display_artist": artist,
                        "duration": text(kid(res, "Duration")),
                        "track_release_reference": tr[0],
                        "track_release_id": tr[1],
                    })

        top = kid(rel, "ResourceGroup")
        if top is not None:
            walk(top, [])
        icpn = rel.xpath("./*[local-name()='ReleaseId']/*[local-name()='ICPN']/text()")
        dan = pick_default(kids(rel, "DisplayArtistName"))
        out.append({
            "release_reference": text(kid(rel, "ReleaseReference")),
            "title": title_of(rel),
            "icpn": icpn[0].strip() if icpn else None,
            "display_artist": text(dan),
            "tracks": tracks,
        })
    deal_refs = [t.strip() for t in root.xpath(
        "./*[local-name()='DealList']/*[local-name()='ReleaseDeal']/*[local-name()='DealReleaseReference']/text()")]
    return {"file": path, "releases": out,
            "track_release_count": len(track_releases),
            "deal_release_references": sorted(set(deal_refs))}


if __name__ == "__main__":
    json.dump([oracle(p) for p in sys.argv[1:]], sys.stdout, indent=2, ensure_ascii=False)
    print()
