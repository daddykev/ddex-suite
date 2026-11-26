"""DDEXParser.sanity_check on official samples and on bad input.

Official samples: DDEX's ERN 4.2 and 4.3 samples (examples/Samples42, Samples43) and
YouTube's published ERN 3.8.2 samples (samples/youtube-ern382, fetched with
scripts/fetch_ern382_samples.sh when missing, as the Rust tests do; DDEX_SKIP_SAMPLES=1
skips them). The expected version comes from the root element's namespace, read with
ElementTree, independently of the parser.

Run (with the extension built, e.g. `maturin develop`):
    python -m pytest tests/test_sanity_check.py
"""

import glob
import os
import subprocess
import xml.etree.ElementTree as ET

import pytest
from ddex_parser import DDEXParser

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", ".."))
NAMESPACE_VERSIONS = {
    "http://ddex.net/xml/ern/382": "V3_8_2",
    "http://ddex.net/xml/ern/42": "V4_2",
    "http://ddex.net/xml/ern/43": "V4_3",
}


def youtube_ern382_samples():
    directory = os.path.join(REPO, "samples", "youtube-ern382")
    if not glob.glob(os.path.join(directory, "*.xml")):
        if os.environ.get("DDEX_SKIP_SAMPLES") == "1":
            return []
        subprocess.run(["sh", os.path.join(REPO, "scripts", "fetch_ern382_samples.sh")], check=True)
    return sorted(glob.glob(os.path.join(directory, "*.xml")))


def official_samples():
    ern4 = sorted(
        glob.glob(os.path.join(REPO, "examples", "Samples42", "*.xml"))
        + glob.glob(os.path.join(REPO, "examples", "Samples43", "*.xml"))
    )
    assert ern4, "official ERN 4.x samples not found under examples/"
    return ern4 + youtube_ern382_samples()


def expected_version(path):
    tag = ET.parse(path).getroot().tag  # "{namespace}LocalName"
    return NAMESPACE_VERSIONS[tag[1:].split("}")[0]]


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


SAMPLES = official_samples()


def test_samples_cover_every_version():
    versions = {expected_version(p) for p in SAMPLES}
    if os.environ.get("DDEX_SKIP_SAMPLES") != "1":
        assert versions == {"V3_8_2", "V4_2", "V4_3"}


@pytest.mark.parametrize("path", SAMPLES, ids=lambda p: os.path.relpath(p, REPO))
def test_official_sample_is_valid_with_its_version(path):
    result = DDEXParser().sanity_check(read(path))
    assert result == {
        "is_valid": True,
        "version": expected_version(path),
        "errors": [],
        "warnings": [],
    }


def test_mismatched_end_tag_is_invalid():
    xml = (
        '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">'
        "<MessageHeader></Mismatched></ern:NewReleaseMessage>"
    )
    result = DDEXParser().sanity_check(xml)
    assert result["is_valid"] is False
    assert result["version"] == "V4_3"  # detected from the root namespace
    assert len(result["errors"]) == 1
    assert "Not well-formed XML" in result["errors"][0]


def test_truncated_official_sample_is_invalid():
    path = os.path.join(REPO, "examples", "Samples43", "1 Audio.xml")
    text = read(path)
    result = DDEXParser().sanity_check(text[: len(text) // 2])
    assert result["is_valid"] is False
    assert result["version"] == "V4_3"
    assert any("not closed" in e for e in result["errors"]), result["errors"]


def test_duplicate_attribute_and_undefined_entity_are_reported():
    xml = (
        '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">'
        '<MessageHeader a="1" a="2">&undefined;</MessageHeader></ern:NewReleaseMessage>'
    )
    result = DDEXParser().sanity_check(xml)
    assert result["is_valid"] is False
    assert result["version"] == "V4_3"
    assert any("Invalid attribute" in e for e in result["errors"]), result["errors"]
    assert any("Undefined entity &undefined;" in e for e in result["errors"]), result["errors"]


def test_non_ddex_xml_is_invalid_without_a_version():
    result = DDEXParser().sanity_check('<?xml version="1.0"?><html><body>Hi</body></html>')
    assert result["is_valid"] is False
    assert result["version"] is None
    assert len(result["errors"]) == 1
    assert "No DDEX ERN namespace found" in result["errors"][0]
    assert result["warnings"] == []


@pytest.mark.parametrize("text", ["", "   ", "not xml at all"])
def test_not_xml_is_invalid_without_a_version(text):
    result = DDEXParser().sanity_check(text)
    assert result["is_valid"] is False
    assert result["version"] is None
    assert result["errors"]
