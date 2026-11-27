"""request_from_xml: parse -> modify -> build, on the official samples
(examples/Samples43/1 Audio.xml and 8 DjMix.xml).

Run (with both extensions built, e.g. `maturin develop` here and in
packages/ddex-parser/bindings/python, plus pandas):
    python -m pytest test_request_from_xml.py
"""

import os

import ddex_parser
import pytest

from ddex_builder import DdexBuilder, request_from_xml

# CI runs these tests from a copy outside the repository.
REPO = os.environ.get(
    "GITHUB_WORKSPACE", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")
)


def read(name):
    with open(os.path.join(REPO, "examples", "Samples43", name), encoding="utf-8") as f:
        return f.read()


def summary(xml):
    """Release titles, track ISRCs and deal territories."""
    message = ddex_parser.DDEXParser().parse(xml)
    return (
        list(message.to_dataframe("releases")["title"]),
        list(message.to_dataframe("tracks")["isrc"]),
        [d["territories"] for d in message.deals()],
    )


def diff(where, a, b, out):
    """Every leaf that differs, as `path: a -> b`."""
    if isinstance(a, dict) and isinstance(b, dict):
        for key in dict.fromkeys([*a, *b]):
            diff(f"{where}.{key}", a.get(key), b.get(key), out)
    elif isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
        for i, (x, y) in enumerate(zip(a, b)):
            diff(f"{where}[{i}]", x, y, out)
    elif a != b:
        out.append(f"{where}: {a!r} -> {b!r}")
    return out


def mapped(xml, original=None):
    """The request of `xml`, with the values the builder fills in (deal references, a
    default resource group) cleared where `original` left them out."""
    request = request_from_xml(xml)["request"]
    if original is not None:
        for deal, o in zip(request["deals"], original["deals"]):
            if o.get("deal_reference") is None:
                deal["deal_reference"] = None
        for release, o in zip(request["releases"], original["releases"]):
            if o.get("resource_group") is None:
                release["resource_group"] = None
    return request


def test_build_of_the_mapped_request_keeps_titles_isrcs_and_territories():
    xml = read("1 Audio.xml")
    rebuilt = DdexBuilder().build(request_from_xml(xml)["request"])
    assert summary(rebuilt) == summary(xml)


def test_unmapped_lists_what_the_request_cannot_hold():
    unmapped = request_from_xml(read("1 Audio.xml"))["unmapped"]
    for path in ["header.message_thread_id", "attributes.ReleaseProfileVersionId", "resources[0].p_line"]:
        assert path in unmapped, unmapped


def test_modifying_the_title_changes_only_the_title():
    xml = read("1 Audio.xml")
    modified = request_from_xml(xml)["request"]
    modified["releases"][0]["title"][0]["text"] = "Changed Title"
    changed = DdexBuilder().build(modified)
    diffs = diff("request", mapped(xml), mapped(changed, modified), [])
    assert len(diffs) == 1, diffs
    assert diffs[0].startswith("request.releases[0].title[0].text: ")
    assert diffs[0].endswith("-> 'Changed Title'")


def test_dj_mix_empty_message_id_fails_the_build_named():
    bridge = request_from_xml(read("8 DjMix.xml"))
    assert bridge["request"]["header"]["message_id"] == ""
    with pytest.raises(RuntimeError, match=r"requires MessageHeader/MessageId \(header\.message_id\)"):
        DdexBuilder().build(bridge["request"])


def test_a_parse_error_raises_value_error():
    with pytest.raises(ValueError, match="Parse error"):
        request_from_xml("<not ddex/>")
