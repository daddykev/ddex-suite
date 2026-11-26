//! A reference to an undefined entity (`&foo;`; DTDs are not read, so only the five
//! predefined entities and character references are defined) is an error in every reader,
//! naming the entity and giving its line and column. Before 0.5.1 several readers dropped it
//! silently. The inputs are the official sample `Samples43/1 Audio.xml` with `&foo;`
//! injected into the first sound recording's title text, or into an attribute value.

use ddex_parser::parser::mode::ParseMode;
use ddex_parser::parser::multi_release_parser::MultiReleaseParser;
use ddex_parser::parser::selective_parser::SelectiveParser;
use ddex_parser::parser::xpath_selector::XPathSelector;
use ddex_parser::parser::ParseOptions;
use ddex_parser::{DDEXParser, ERNVersion};
use std::io::Cursor;

const TITLE: &str = "<TitleText>Yume no Lullaby</TitleText>";
const DISPLAY_TITLE: &str =
    r#"<DisplayTitle ApplicableTerritoryCode="Worldwide" IsDefault="true">"#;

/// `&foo;` in text: line 73, column 29 (byte 2809) of the sample.
const TEXT_ERROR: &str = "undefined entity '&foo;' at line 73, column 29";
/// `&foo;` in the first DisplayTitle's `ApplicableTerritoryCode`: line 72, column 54.
const ATTRIBUTE_ERROR: &str =
    "undefined entity '&foo;' in an attribute value at line 72, column 54";

fn sample() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Samples43/1 Audio.xml"
    );
    std::fs::read_to_string(path).expect("official sample examples/Samples43/1 Audio.xml")
}

fn replace_first(xml: &str, from: &str, to: &str) -> String {
    assert!(xml.contains(from), "{from} is in the sample");
    xml.replacen(from, to, 1)
}

fn in_text() -> String {
    replace_first(
        &sample(),
        TITLE,
        "<TitleText>Yume &foo; Lullaby</TitleText>",
    )
}

fn in_attribute() -> String {
    replace_first(
        &sample(),
        DISPLAY_TITLE,
        r#"<DisplayTitle ApplicableTerritoryCode="World&foo;wide" IsDefault="true">"#,
    )
}

fn parse(xml: &str) -> Result<ddex_core::models::flat::ParsedERNMessage, String> {
    DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .map_err(|e| e.to_string())
}

#[test]
fn parse_fails_on_an_undefined_entity_in_text() {
    let error = parse(&in_text()).unwrap_err();
    assert_eq!(
        error,
        format!("Malformed XML at position 2809: {TEXT_ERROR}")
    );
}

#[test]
fn parse_fails_on_an_undefined_entity_in_an_attribute() {
    let error = parse(&in_attribute()).unwrap_err();
    assert!(error.ends_with(ATTRIBUTE_ERROR), "{error}");
}

#[test]
fn predefined_entities_and_character_references_still_decode() {
    let decoded = "A & B café café";
    let xml = sample()
        .replace(
            "<DisplayTitleText>Yume no Lullaby</DisplayTitleText>",
            "<DisplayTitleText>A &amp; B caf&#233; caf&#xE9;</DisplayTitleText>",
        )
        .replace(
            TITLE,
            "<TitleText>A &amp; B caf&#233; caf&#xE9;</TitleText>",
        );
    let xml = replace_first(
        &xml,
        r#"ReleaseProfileVersionId="Audio""#,
        r#"ReleaseProfileVersionId="A&amp;u&#233;&#xE9;""#,
    );
    let message = parse(&xml).unwrap();
    let track = &message.flat.releases[0].tracks[0];
    assert_eq!(track.isrc.as_deref(), Some("JPTO09404900"));
    assert_eq!(track.title.as_deref(), Some(decoded));
    let profile = message
        .graph
        .attributes
        .as_ref()
        .and_then(|attributes| attributes.get_str("ReleaseProfileVersionId"))
        .map(|value| value.to_xml_value());
    assert_eq!(profile.as_deref(), Some("A&uéé"));
}

#[test]
fn experimental_stream_mode_fails() {
    for (xml, expected) in [(in_text(), TEXT_ERROR), (in_attribute(), ATTRIBUTE_ERROR)] {
        let options = ParseOptions {
            mode: ParseMode::Stream,
            ..Default::default()
        };
        let error = DDEXParser::new()
            .parse_with_options(Cursor::new(xml.as_bytes()), options)
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn streaming_iterators_fail() {
    for (xml, expected) in [(in_text(), TEXT_ERROR), (in_attribute(), ATTRIBUTE_ERROR)] {
        let parser = DDEXParser::new();
        let error = parser
            .stream(Cursor::new(xml.as_bytes()))
            .find_map(Result::err)
            .expect("the stream yields an error")
            .to_string();
        assert!(error.contains(expected), "stream: {error}");

        let error = parser
            .stream_parallel(Cursor::new(xml.as_bytes()))
            .find_map(Result::err)
            .expect("the parallel stream yields an error")
            .to_string();
        assert!(error.contains(expected), "stream_parallel: {error}");
    }
}

#[test]
fn selective_readers_fail() {
    for (xml, expected) in [(in_text(), TEXT_ERROR), (in_attribute(), ATTRIBUTE_ERROR)] {
        let bytes = xml.as_bytes();
        let checks: [(&str, Result<(), String>); 5] = [
            (
                "SelectiveParser::extract_isrcs",
                SelectiveParser::for_isrcs()
                    .extract_isrcs(Cursor::new(bytes))
                    .map(drop)
                    .map_err(|e| e.to_string()),
            ),
            (
                "SelectiveParser::extract_isrcs_fast",
                SelectiveParser::for_isrcs()
                    .extract_isrcs_fast(Cursor::new(bytes))
                    .map(drop)
                    .map_err(|e| e.to_string()),
            ),
            (
                "XPathSelector::select",
                XPathSelector::ddex_isrcs()
                    .unwrap()
                    .select(Cursor::new(bytes))
                    .map(drop)
                    .map_err(|e| e.to_string()),
            ),
            (
                "MultiReleaseParser::parse_releases",
                MultiReleaseParser::new(ERNVersion::V4_3)
                    .parse_releases(Cursor::new(bytes))
                    .map(drop)
                    .map_err(|e| e.to_string()),
            ),
            (
                "MultiReleaseParser::count_releases",
                MultiReleaseParser::new(ERNVersion::V4_3)
                    .count_releases(Cursor::new(bytes))
                    .map(drop)
                    .map_err(|e| e.to_string()),
            ),
        ];
        for (reader, result) in checks {
            let error = result.expect_err(reader);
            assert!(error.contains(expected), "{reader}: {error}");
        }
    }
}

#[test]
fn sanity_check_reports_the_entity_with_its_line_and_column() {
    let parser = DDEXParser::new();
    let text = parser
        .sanity_check(Cursor::new(in_text().into_bytes()))
        .unwrap();
    assert!(!text.is_valid);
    assert!(
        text.errors
            .contains(&"Undefined entity '&foo;' at line 73, column 29".to_string()),
        "{:?}",
        text.errors
    );

    let attribute = parser
        .sanity_check(Cursor::new(in_attribute().into_bytes()))
        .unwrap();
    assert!(!attribute.is_valid);
    assert_eq!(
        attribute.errors,
        ["Undefined entity '&foo;' in an attribute value at line 72, column 54"],
    );
}
