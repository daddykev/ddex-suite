"""Smoke test for the 0.4.6 parser fields through the Python binding: the deals() and
message_control_type() accessors and the new DataFrame columns (release identifiers,
dates, P/C lines, genres, parental warnings, labels, 3.8.x track artists and layout,
TrackRelease genre). Absent values are None.

Run (with the extension built, e.g. `maturin develop`, plus pandas):
    python -m pytest tests/test_parser_fields.py
"""

import os

import ddex_parser

FIXTURES = os.path.join(os.path.dirname(__file__), "..", "..", "..", "tests", "fixtures")


def parse(name):
    with open(os.path.join(FIXTURES, name), encoding="utf-8") as f:
        return ddex_parser.DDEXParser().parse(f.read())


def test_ern43_fields():
    parsed = parse("ern43_details.xml")
    assert parsed.message_control_type() == "TestMessage"
    assert parsed.deals() == [
        {
            "deal_id": None,
            "releases": ["R0"],
            "start_date": "2024-09-20T00:00:00+00:00",
            "end_date": "2025-09-19T00:00:00+00:00",
            "territories": ["Worldwide"],
            "excluded_territories": [],
            "usage_rights": ["OnDemandStream"],
            "distribution_channels": [],
        }
    ]

    release = parsed.to_dataframe("releases").to_dict("records")[0]
    assert release["icpn"] == "123456789012"
    assert release["isrc"] is None
    assert release["label_name"] == "Details Records"
    assert (release["genre"], release["sub_genre"]) == ("Pop", "Synth-pop")
    assert [(g["genre"], g["territory_codes"], g["is_default"]) for g in release["genres"]] == [
        ("J-Pop", ["JP"], None),
        ("Pop", [], True),
    ]
    assert release["parental_warning"] == "Explicit"
    assert release["parental_warnings"] == ["Explicit"]
    assert (release["p_line"], release["p_line_year"]) == ("(P) 2024 Details Records", 2024)
    assert release["c_line"] == "(C) 2024 Details Records"
    assert release["release_date"] == "2024-09-20T00:00:00+00:00"
    assert [(d["date"], d["territory_codes"]) for d in release["release_dates"]] == [
        ("2024-09-18", ["JP"]),
        ("2024-09-20", []),
    ]
    # A partial date has no full-date value; the list keeps it as written
    assert release["original_release_date"] is None
    assert [d["date"] for d in release["original_release_dates"]] == ["2016"]

    track = parsed.to_dataframe("tracks").to_dict("records")[0]
    assert track["parental_warning"] == "ExplicitContentEdited"
    assert track["p_line"] == "(P) 2024 Details Records"
    assert track["genre"] is None  # 4.x resources carry no genre
    assert (track["track_release_genre"], track["track_release_sub_genre"]) == ("Pop", "Electropop")

    flat = parsed.to_dataframe("flat").to_dict("records")
    assert flat[0]["message_control_type"] == "TestMessage"
    assert flat[1]["label_name"] == "Details Records"
    assert flat[1]["message_control_type"] is None


def test_ern382_fields():
    parsed = parse("ern382_details.xml")
    assert parsed.message_control_type() == "TestMessage"
    assert parsed.deals()[0]["end_date"] == "2025-09-19T00:00:00+00:00"

    release = parsed.to_dataframe("releases").to_dict("records")[0]
    assert release["proprietary_ids"] == [{"namespace": "DPID:PADPIDA0000000001", "value": "REL-0001"}]
    assert release["label_name"] == "Details Records"
    assert release["genre"] == "Pop"
    assert release["parental_warning"] == "Explicit"
    assert release["c_line_year"] == 2024
    assert [(d["date"], d["territory_codes"]) for d in release["release_dates"]] == [
        ("2024-09-18", ["JP"]),
        ("2024-09-20", ["Worldwide"]),
    ]

    tracks = parsed.to_dataframe("tracks").to_dict("records")
    assert tracks[0]["artist"] == "Details Artist feat. Guest Artist"
    assert [(a["name"], a["role"]) for a in tracks[0]["artists"]] == [
        ("Details Artist", "MainArtist"),
        ("Guest Artist", "FeaturedArtist"),
    ]
    assert [t["track_number"] for t in tracks] == [1, 2]
    assert tracks[0]["group_path"] == [{"group_type": None, "sequence_number": 1}]
    assert (tracks[0]["genre"], tracks[0]["label_name"]) == ("Pop", "Details Records")
    assert tracks[1]["p_line"] is None
    assert tracks[1]["parental_warning"] == "NotExplicit"


def test_parser_to_dataframe_matches_message_to_dataframe():
    with open(os.path.join(FIXTURES, "ern382_details.xml"), encoding="utf-8") as f:
        xml = f.read()
    direct = ddex_parser.DDEXParser().to_dataframe(xml, "releases").to_dict("records")
    via_message = ddex_parser.DDEXParser().parse(xml).to_dataframe("releases").to_dict("records")
    assert direct == via_message
