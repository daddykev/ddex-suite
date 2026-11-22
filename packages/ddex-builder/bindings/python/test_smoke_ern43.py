"""Smoke test: the Python binding builds ERN 4.3 with the core builder, and ddex-parser reads
it back.

Run (with both extensions built, e.g. `maturin develop` here and in
packages/ddex-parser/bindings/python, plus pandas):
    python -m pytest test_smoke_ern43.py
"""

import copy

import pandas as pd
import pytest

import ddex_builder
import ddex_parser
from ddex_builder import DdexBuilder, Release, Resource, batch_build

HEADER = {
    "message_id": "PY-SMOKE-1",
    "message_sender": {"party_name": [{"text": "Smoke Label"}], "party_id": "PADPIDA0000000001"},
    "message_recipient": {"party_name": [{"text": "Smoke DSP"}], "party_id": "PADPIDA0000000002"},
    "message_created_date_time": "2024-09-13T12:00:00Z",
}
MESSAGE = {"header": HEADER, "version": "4.3", "language_and_script_code": "en", "avs_version_id": "12"}

REQUEST = dict(
    MESSAGE,
    releases=[
        {
            "release_id": "A1-ABCDE-0000000001-M",
            "release_reference": "R0",
            "title": [{"text": "Smoke & \"Mirrors\" <Live>"}],
            "artist": "Smoke Artist",
            "label": "Smoke Records",
            "upc": "123456789012",
            "release_type": "Album",
            "genre": "Pop",
            "parental_warning_type": "NotExplicit",
            "tracks": [
                {
                    "track_id": f"T{n}",
                    "resource_reference": f"A{n}",
                    "isrc": f"USRT1240000{n}",
                    "title": f"Track {n}",
                    "duration": "PT3M42S",
                    "artist": "Smoke Artist",
                    "resource_type": "MusicalWorkSoundRecording",
                    "parental_warning_type": "NotExplicit",
                }
                for n in (1, 2)
            ],
        }
    ],
    deals=[
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
)


def parse_tracks(xml):
    parsed = ddex_parser.DDEXParser().parse(xml)
    return parsed.to_dataframe("tracks").to_dict("records")


def test_native_core_builder_is_used():
    assert ddex_builder.is_rust_available()


def test_build_request_round_trips_through_the_parser():
    xml = DdexBuilder().build(REQUEST)
    assert 'xmlns:ern="http://ddex.net/xml/ern/43"' in xml
    assert "<SoundRecordingEdition>" in xml
    tracks = parse_tracks(xml)
    assert [(t["isrc"], t["track_title"]) for t in tracks] == [
        ("USRT12400001", "Track 1"),
        ("USRT12400002", "Track 2"),
    ]
    assert tracks[0]["release_title"] == 'Smoke & "Mirrors" <Live>'


def test_stored_releases_with_request_header():
    builder = DdexBuilder()
    builder.add_release(
        Release(
            "A1-ABCDE-0000000002-M", "Single", "Stored Single", "Smoke Artist",
            label="Smoke Records", genre="Pop", track_ids=["A9"],
            parental_warning_type="NotExplicit",
        )
    )
    builder.add_resource(
        Resource(
            "A9", "SoundRecording", "Stored Track", "Smoke Artist",
            isrc="USRT12400009", duration="PT2M30S",
            ddex_type="MusicalWorkSoundRecording", parental_warning_type="NotExplicit",
        )
    )
    tracks = parse_tracks(builder.build(MESSAGE))
    assert [t["track_title"] for t in tracks] == ["Stored Track"]


def test_from_dataframe_uses_the_core_builder():
    df = pd.DataFrame(
        [
            {
                "release_id": "A1-ABCDE-0000000003-M",
                "release_reference": "R0",
                "release_title": "Frame Album",
                "release_artist": "Frame Artist",
                "release_label": "Frame Records",
                "release_genre": "Pop",
                "release_type": "Album",
                "release_parental_warning_type": "NotExplicit",
                "track_index": i,
                "track_title": f"Frame {i + 1}",
                "artist": "Frame Artist",
                "isrc": f"USRT1240010{i}",
                "duration": 201.0,  # seconds, as ddex-parser's DataFrames carry them
                "resource_type": "MusicalWorkSoundRecording",
                "parental_warning_type": "NotExplicit",
            }
            for i in range(3)
        ]
    )
    xml = DdexBuilder().from_dataframe(df, schema="tracks", request=MESSAGE)
    tracks = parse_tracks(xml)
    assert [t["track_title"] for t in tracks] == ["Frame 1", "Frame 2", "Frame 3"]
    assert [t["duration"] for t in tracks] == [201.0, 201.0, 201.0]


def test_batch_build_uses_the_core_builder():
    other = copy.deepcopy(REQUEST)
    other["header"]["message_id"] = "PY-SMOKE-2"
    xmls = batch_build([REQUEST, other])
    assert len(xmls) == 2
    assert "<MessageId>PY-SMOKE-2</MessageId>" in xmls[1]


def test_missing_values_are_errors_not_defaults():
    with pytest.raises(ValueError, match="missing field `version`"):
        DdexBuilder().build({"header": HEADER})
    incomplete = copy.deepcopy(REQUEST)
    del incomplete["releases"][0]["genre"]
    with pytest.raises(RuntimeError, match=r"ERN 4\.3 requires Release/Genre/GenreText \(releases\[0\]\.genre\)"):
        DdexBuilder().build(incomplete)
