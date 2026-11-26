"""A reference to an undefined entity (`&foo;`; DTDs are not read) makes parse() raise an
error that names the entity and gives its line and column, in text and in attribute values.
The predefined entities and character references still decode. Input: the official sample
examples/Samples43/1 Audio.xml with `&foo;` injected into the first sound recording's title.

Run (with the extension built, e.g. `maturin develop`):
    python -m pytest tests/test_undefined_entities.py
"""

import os

import pytest

import ddex_parser

SAMPLE = os.path.join(
    os.path.dirname(__file__), "..", "..", "..", "..", "..", "examples", "Samples43", "1 Audio.xml"
)
TITLE = "<TitleText>Yume no Lullaby</TitleText>"
DISPLAY_TITLE = '<DisplayTitle ApplicableTerritoryCode="Worldwide" IsDefault="true">'


def sample():
    with open(SAMPLE, encoding="utf-8") as f:
        xml = f.read()
    assert TITLE in xml and DISPLAY_TITLE in xml
    return xml


def test_undefined_entity_in_text_raises_with_its_name_line_and_column():
    xml = sample().replace(TITLE, "<TitleText>Yume &foo; Lullaby</TitleText>", 1)
    with pytest.raises(ValueError) as error:
        ddex_parser.DDEXParser().parse(xml)
    assert "undefined entity '&foo;' at line 73, column 29" in str(error.value)


def test_undefined_entity_in_an_attribute_raises_with_its_name_line_and_column():
    xml = sample().replace(
        DISPLAY_TITLE,
        '<DisplayTitle ApplicableTerritoryCode="World&foo;wide" IsDefault="true">',
        1,
    )
    with pytest.raises(ValueError) as error:
        ddex_parser.DDEXParser().parse(xml)
    assert "undefined entity '&foo;' in an attribute value at line 72, column 54" in str(
        error.value
    )


def test_predefined_entities_and_character_references_decode():
    decoded = "A &amp; B caf&#233; caf&#xE9;"
    xml = (
        sample()
        .replace(
            "<DisplayTitleText>Yume no Lullaby</DisplayTitleText>",
            f"<DisplayTitleText>{decoded}</DisplayTitleText>",
        )
        .replace(TITLE, f"<TitleText>{decoded}</TitleText>")
    )
    tracks = ddex_parser.DDEXParser().parse(xml).to_dataframe("tracks").to_dict("records")
    track = next(t for t in tracks if t["isrc"] == "JPTO09404900")
    assert track["track_title"] == "A & B café café"
