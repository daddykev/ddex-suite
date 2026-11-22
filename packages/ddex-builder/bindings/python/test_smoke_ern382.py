"""Smoke test: the Python binding builds ERN 3.8.2 (with deals) with the core builder, and
ddex-parser reads it back.

Run (with both extensions built, e.g. `maturin develop` here and in
packages/ddex-parser/bindings/python, plus pandas):
    python -m pytest test_smoke_ern382.py
"""

import copy

import pytest

import ddex_parser
from ddex_builder import DdexBuilder

REQUEST = {
    "header": {
        "message_id": "PY-SMOKE-382",
        "message_sender": {"party_name": [{"text": "Smoke Label"}], "party_id": "PADPIDA0000000001"},
        "message_recipient": {"party_name": [{"text": "Smoke DSP"}], "party_id": "PADPIDA0000000002"},
        "message_created_date_time": "2024-09-13T12:00:00Z",
    },
    "version": "3.8.2",
    "releases": [
        {
            "release_id": "A1-ABCDE-0000000001-M",
            "release_reference": "R0",
            "title": [{"text": "Smoke & \"Mirrors\" <Live>"}],
            "artist": "Smoke Artist",
            "label": "Smoke Records",
            "upc": "123456789012",
            "release_type": "Album",
            "tracks": [
                {
                    "track_id": f"T{n}",
                    "resource_reference": f"A{n}",
                    "isrc": f"USRT1240000{n}",
                    "title": f"Track {n}",
                    "duration": "PT3M42S",
                    "artist": "Smoke Artist",
                }
                for n in (1, 2)
            ],
        }
    ],
    "deals": [
        {
            "deal_terms": {
                "commercial_model_type": "SubscriptionModel",
                "territory_code": ["Worldwide"],
                "start_date": "2024-01-01",
                "use_types": ["OnDemandStream"],
            },
            "release_references": ["R0"],
        }
    ],
}


def test_ern382_build_round_trips_through_the_parser():
    xml = DdexBuilder().build(REQUEST)
    assert 'xmlns:ern="http://ddex.net/xml/ern/382"' in xml
    assert "<SoundRecordingId>" in xml
    assert "<ReleaseDeal>" in xml
    parsed = ddex_parser.DDEXParser().parse(xml)
    tracks = parsed.to_dataframe("tracks").to_dict("records")
    assert [(t["isrc"], t["track_title"]) for t in tracks] == [
        ("USRT12400001", "Track 1"),
        ("USRT12400002", "Track 2"),
    ]
    assert tracks[0]["release_title"] == 'Smoke & "Mirrors" <Live>'
    assert parsed.release_count() == 1
    # The Python parser binding does not expose deals; check the DealList in the XML
    assert xml.count("<DealReleaseReference>R0</DealReleaseReference>") == 1


def test_values_the_382_schema_requires_are_errors():
    incomplete = copy.deepcopy(REQUEST)
    del incomplete["deals"][0]["deal_terms"]["use_types"]
    with pytest.raises(
        RuntimeError,
        match=r"ERN 3\.8\.2 requires DealTerms/Usage/UseType \(deals\[0\]\.deal_terms\.use_types\)",
    ):
        DdexBuilder().build(incomplete)
