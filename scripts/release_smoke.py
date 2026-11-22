#!/usr/bin/env python3
"""Install test for the release packages: import ddex_parser and ddex_builder (each fails to
import without its native extension), parse an official sample, and build a small
ERN 4.3 message that the parser reads back.

Usage: python scripts/release_smoke.py "examples/Samples43/1 Audio.xml"
Run it from outside the package source directories, so the installed packages are used.
"""

import sys
from importlib.metadata import version

import ddex_builder
import ddex_parser


def main(sample_path):
    assert ddex_parser.__version__ == version("ddex-parser"), ddex_parser.__version__
    assert ddex_builder.__version__ == version("ddex-builder"), ddex_builder.__version__

    with open(sample_path, encoding="utf-8") as f:
        parsed = ddex_parser.DDEXParser().parse(f.read())
    assert parsed.release_count() >= 1, "no release parsed"
    print(f"parsed {sample_path}: {parsed.release_count()} release(s), {len(parsed.deals())} deal(s)")

    party = lambda name, pid: {"party_name": [{"text": name}], "party_id": pid}  # noqa: E731
    request = {
        "header": {
            "message_id": "RELEASE-SMOKE",
            "message_sender": party("Smoke Label", "PADPIDA0000000001"),
            "message_recipient": party("Smoke DSP", "PADPIDA0000000002"),
            "message_created_date_time": "2024-09-13T12:00:00Z",
        },
        "version": "4.3",
        "avs_version_id": "12",
        "language_and_script_code": "en",
        "releases": [
            {
                "release_id": "A1-ABCDE-0000000001-M",
                "title": [{"text": "Smoke Single"}],
                "artist": "Smoke Artist",
                "label": "Smoke Label",
                "release_type": "Single",
                "genre": "Pop",
                "parental_warning_type": "NotExplicit",
                "tracks": [
                    {
                        "track_id": "T1",
                        "isrc": "USRT12400001",
                        "title": "Smoke Track",
                        "duration": "PT3M42S",
                        "artist": "Smoke Artist",
                        "resource_type": "MusicalWorkSoundRecording",
                        "parental_warning_type": "NotExplicit",
                    }
                ],
            }
        ],
        "deals": [],
    }
    xml = ddex_builder.DdexBuilder().build(request)
    rebuilt = ddex_parser.DDEXParser().parse(xml)
    assert rebuilt.message_id() == "RELEASE-SMOKE"
    assert rebuilt.release_count() == 1
    print(f"built {len(xml)} bytes of ERN 4.3 and parsed it back")
    print(f"ok - ddex-parser {ddex_parser.__version__}, ddex-builder {ddex_builder.__version__}")


if __name__ == "__main__":
    main(sys.argv[1])
