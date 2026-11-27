//! XML 1.0 §3.3.3 attribute-value normalization (CDATA attributes, no DTD) on the public
//! readers that return attribute values, against the official sample
//! `Samples43/1 Audio.xml` with one attribute added:
//!
//! - a literal tab, LF, CRLF and lone CR each become one space (nothing is collapsed or
//!   trimmed);
//! - the character references `&#9;`, `&#10;`, `&#13;` are not normalized: the readers that
//!   resolve references give the actual character, the ones that return values as written
//!   keep the reference text.
//!
//! The lxml oracle job compares flat fields, not attribute values, so the expected values
//! here are the ones the spec gives. The other readers (`detect_version`, `stream`,
//! `ParseMode::Stream`, selective, multi-release, extension capture, zero-copy) do not
//! return attribute values of this kind; they apply the same normalization internally.

use ddex_parser::parser::xpath_selector::XPathSelector;
use ddex_parser::DDEXParser;
use std::io::Cursor;

/// The value as written in the fixture: tab, LF, CRLF, lone CR, then the three character
/// references.
const RAW: &str = "a\tb\nc\r\nd\re&#9;f&#10;g&#13;h";
/// After normalization and reference resolution.
const RESOLVED: &str = "a b c d e\tf\ng\rh";
/// After normalization only (readers that keep references as written).
const AS_WRITTEN: &str = "a b c d e&#9;f&#10;g&#13;h";

fn sample() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Samples43/1 Audio.xml"
    );
    std::fs::read_to_string(path).expect("official sample examples/Samples43/1 Audio.xml")
}

/// The sample with `Test="<RAW>"` on the root element and on the first `<ISRC>`.
fn fixture() -> String {
    let xml = sample();
    assert!(xml.contains("<ern:NewReleaseMessage") && xml.contains("<ISRC>"));
    xml.replacen(
        "<ern:NewReleaseMessage",
        &format!(r#"<ern:NewReleaseMessage Test="{RAW}""#),
        1,
    )
    .replacen("<ISRC>", &format!(r#"<ISRC Test="{RAW}">"#), 1)
}

#[test]
fn dom_parse_normalizes_and_resolves_attribute_values() {
    let parsed = DDEXParser::new()
        .parse(Cursor::new(fixture().into_bytes()))
        .expect("parse");
    let attributes = parsed.graph.attributes.expect("root attributes");
    let value = attributes.get_str("Test").expect("Test attribute");
    assert_eq!(value.to_xml_value(), RESOLVED);
}

#[test]
fn xpath_normalizes_attribute_values_as_written() {
    let result = XPathSelector::new("//ISRC")
        .unwrap()
        .select(Cursor::new(fixture().into_bytes()))
        .expect("select");
    let first = result
        .attributes
        .iter()
        .find_map(|attributes| attributes.get("Test"))
        .expect("Test attribute on the first ISRC");
    assert_eq!(first, AS_WRITTEN);
}

#[test]
fn a_value_without_literal_whitespace_is_unchanged() {
    let xml = sample().replacen(
        "<ern:NewReleaseMessage",
        r#"<ern:NewReleaseMessage Test="x &amp; y&#9;z""#,
        1,
    );
    let parsed = DDEXParser::new()
        .parse(Cursor::new(xml.into_bytes()))
        .expect("parse");
    let attributes = parsed.graph.attributes.expect("root attributes");
    assert_eq!(
        attributes.get_str("Test").unwrap().to_xml_value(),
        "x & y\tz"
    );
}
