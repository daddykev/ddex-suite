"""validate_structure reads the whole document: a reference to an undefined entity
(`&foo;`; DTDs are not read) makes it invalid, with the entity's name, line and column, and
so does a syntax error after the first element (before 0.5.1 only the first event was read).
Input: the official sample examples/Samples43/1 Audio.xml.

Run (with the extension built, e.g. `maturin develop`):
    python -m pytest test_validate_structure.py
"""

import os

import ddex_builder

# CI runs these tests from a copy outside the repository.
REPO = os.environ.get(
    "GITHUB_WORKSPACE", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..")
)
SAMPLE = os.path.join(REPO, "examples", "Samples43", "1 Audio.xml")
TITLE = "<TitleText>Yume no Lullaby</TitleText>"
DISPLAY_TITLE = '<DisplayTitle ApplicableTerritoryCode="Worldwide" IsDefault="true">'


def sample():
    with open(SAMPLE, encoding="utf-8") as f:
        xml = f.read()
    assert TITLE in xml and DISPLAY_TITLE in xml
    return xml


def test_the_official_sample_is_valid():
    result = ddex_builder.validate_structure(sample())
    assert result.is_valid, result.errors
    assert result.errors == []


def test_undefined_entity_in_text():
    xml = sample().replace(TITLE, "<TitleText>Yume &foo; Lullaby</TitleText>", 1)
    result = ddex_builder.validate_structure(xml)
    assert not result.is_valid
    assert result.errors == ["XML parsing error: undefined entity '&foo;' at line 73, column 29"]


def test_undefined_entity_in_an_attribute():
    xml = sample().replace(
        DISPLAY_TITLE,
        '<DisplayTitle ApplicableTerritoryCode="World&foo;wide" IsDefault="true">',
        1,
    )
    result = ddex_builder.validate_structure(xml)
    assert not result.is_valid
    assert result.errors == [
        "XML parsing error: undefined entity '&foo;' in an attribute value at line 72, column 54"
    ]


def test_a_mismatched_end_tag_after_the_first_element():
    xml = sample().replace("</TitleText>", "</Title>", 1)
    result = ddex_builder.validate_structure(xml)
    assert not result.is_valid
    assert len(result.errors) == 1 and result.errors[0].startswith("XML parsing error:")
