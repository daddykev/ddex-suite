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

`details()` computes the 0.5.0 fields independently: MessageControlType; per main Release
the default and full lists of genres, parental warnings, release / original release
dates, the default P/C line and the label (first ReleaseLabelReference -> PartyList name);
per track the default parental warning, P/C line (4.3: in the Edition) and TrackRelease
genre; per Deal the ValidityPeriod start / end. Defaults: IsDefault="true", else an entry
without ApplicableTerritoryCode (or Worldwide), else the first.
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


# ---------------------------------------------------------------------------
# 0.5.0 fields: dates, P/C lines, genres, parental warnings, label, MessageControlType,
# TrackRelease genre, deal dates. Text is taken exactly as written (lxml text, no strip),
# as the parser keeps xs:string values; references are collapsed (xs:IDREF).
# ---------------------------------------------------------------------------

def raw(el):
    """Text of a leaf element exactly as written ("" when empty), None when absent."""
    if el is None:
        return None
    return el.text or ""


def ref(el):
    """A reference (xs:IDREF): whitespace collapsed."""
    v = raw(el)
    return " ".join(v.split()) if v is not None else None


def default_of(els):
    """IsDefault="true", else the first without ApplicableTerritoryCode or with Worldwide,
    else the first."""
    for e in els:
        if e.get("IsDefault") == "true":
            return e
    for e in els:
        if e.get("ApplicableTerritoryCode") in (None, "Worldwide"):
            return e
    return els[0] if els else None


def full_date(v):
    """A full YYYY-MM-DD date as the parser's RFC 3339 midnight UTC; None otherwise."""
    if v is None:
        return None
    v = v.strip()
    if len(v) == 10 and v[4] == "-" and v[7] == "-" and v.replace("-", "").isdigit():
        return v + "T00:00:00Z"
    return None


def year(el):
    v = raw(el)
    try:
        return int(v.strip()) if v is not None else None
    except ValueError:
        return None


def line(el, kind):
    if el is None:
        return None
    return {"text": raw(kid(el, kind + "LineText")), "year": year(kid(el, "Year"))}


def genre_list(parent):
    return [{"genre": raw(kid(g, "GenreText")), "sub_genre": raw(kid(g, "SubGenre")),
             "territory": g.get("ApplicableTerritoryCode"),
             "is_default": {"true": True, "false": False}.get(g.get("IsDefault"))}
            for g in kids(parent, "Genre") if kid(g, "GenreText") is not None]


def dates(parent, name):
    return [{"date": raw(d), "territory": d.get("ApplicableTerritoryCode")}
            for d in kids(parent, name)]


def details(path):
    root = etree.parse(path).getroot()
    header = kid(root, "MessageHeader")
    parties = {}
    for p in root.xpath("./*[local-name()='PartyList']/*[local-name()='Party']"):
        names = kids(p, "PartyName")
        parties[ref(kid(p, "PartyReference"))] = raw(kid(names[0], "FullName")) if names else None
    resources = {ref(kid(r, "ResourceReference")): r
                 for r in root.xpath("./*[local-name()='ResourceList']/*")}
    tr_genre = {}
    for tr in root.xpath("./*[local-name()='ReleaseList']/*[local-name()='TrackRelease']"):
        g = default_of([x for x in kids(tr, "Genre") if kid(x, "GenreText") is not None])
        tr_genre.setdefault(ref(kid(tr, "ReleaseResourceReference")),
                            (raw(kid(g, "GenreText")), raw(kid(g, "SubGenre"))) if g is not None else (None, None))

    def resource_lines(res, kind):
        els = kids(res, kind + "Line")
        for ed in res:
            if ln(ed.tag) in ("SoundRecordingEdition", "VideoEdition"):
                els += kids(ed, kind + "Line")
        return line(default_of([e for e in els if kid(e, kind + "LineText") is not None]), kind)

    releases = []
    for rel in root.xpath("./*[local-name()='ReleaseList']/*[local-name()='Release']"):
        tracks = []

        def walk(group):
            for c in group:
                if ln(c.tag) == "ResourceGroup":
                    walk(c)
                elif ln(c.tag) == "ResourceGroupContentItem":
                    rr = ref(kid(c, "ReleaseResourceReference"))
                    res = resources.get(rr)
                    if res is None or ln(res.tag) not in ("SoundRecording", "Video"):
                        continue
                    pw = default_of(kids(res, "ParentalWarningType"))
                    g = tr_genre.get(rr, (None, None))
                    tracks.append({"ref": rr, "parental_warning": raw(pw),
                                   "p_line": resource_lines(res, "P"),
                                   "c_line": resource_lines(res, "C"),
                                   "track_release_genre": g[0], "track_release_sub_genre": g[1]})

        top = kid(rel, "ResourceGroup")
        if top is not None:
            walk(top)
        genres = [g for g in kids(rel, "Genre") if kid(g, "GenreText") is not None]
        g = default_of(genres)
        pws = kids(rel, "ParentalWarningType")
        label_refs = kids(rel, "ReleaseLabelReference")
        rd = default_of(kids(rel, "ReleaseDate"))
        ord_ = default_of(kids(rel, "OriginalReleaseDate"))
        releases.append({
            "genre": raw(kid(g, "GenreText")) if g is not None else None,
            "sub_genre": raw(kid(g, "SubGenre")) if g is not None else None,
            "genres": genre_list(rel),
            "parental_warning": raw(default_of(pws)),
            "parental_warnings": [raw(w) for w in pws],
            "label_name": parties.get(ref(label_refs[0])) if label_refs else None,
            "p_line": line(default_of([e for e in kids(rel, "PLine") if kid(e, "PLineText") is not None]), "P"),
            "c_line": line(default_of([e for e in kids(rel, "CLine") if kid(e, "CLineText") is not None]), "C"),
            "release_date": full_date(raw(rd)),
            "release_dates": dates(rel, "ReleaseDate"),
            "original_release_date": full_date(raw(ord_)),
            "original_release_dates": dates(rel, "OriginalReleaseDate"),
            "tracks": tracks,
        })

    deals = []
    for rd in root.xpath("./*[local-name()='DealList']/*[local-name()='ReleaseDeal']"):
        for d in kids(rd, "Deal"):
            periods = d.xpath("./*[local-name()='DealTerms']/*[local-name()='ValidityPeriod']")
            starts = [full_date(raw(kid(p, "StartDate"))) for p in periods if kid(p, "StartDate") is not None]
            ends = [full_date(raw(kid(p, "EndDate"))) for p in periods if kid(p, "EndDate") is not None]
            deals.append({"start": starts[-1] if starts else None, "end": ends[-1] if ends else None})
    return {"file": path,
            "message_control_type": raw(kid(header, "MessageControlType")),
            "releases": releases, "deals": deals}


if __name__ == "__main__":
    json.dump([{**oracle(p), "details": details(p)} for p in sys.argv[1:]], sys.stdout,
              indent=2, ensure_ascii=False)
    print()
