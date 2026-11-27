//! The builder's XML readers (DB-C14N canonicalization and version conversion of an input
//! document) fail on a reference to an undefined entity (`&foo;`; DTDs are not read), naming
//! it with its line and column, instead of dropping or copying it. Input: the official
//! sample `Samples43/1 Audio.xml` with `&foo;` injected into a text node or an attribute.

use ddex_builder::versions::{ConverterResult, VersionConverter};
use ddex_builder::{DdexVersion, DB_C14N};

const TITLE: &str = "<TitleText>Yume no Lullaby</TitleText>";
const DISPLAY_TITLE: &str =
    r#"<DisplayTitle ApplicableTerritoryCode="Worldwide" IsDefault="true">"#;
const TEXT_ERROR: &str = "undefined entity '&foo;' at line 73, column 29";
const ATTRIBUTE_ERROR: &str =
    "undefined entity '&foo;' in an attribute value at line 72, column 54";

fn sample() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Samples43/1 Audio.xml"
    );
    std::fs::read_to_string(path).expect("official sample examples/Samples43/1 Audio.xml")
}

fn injected() -> [(String, &'static str); 2] {
    let xml = sample();
    assert!(xml.contains(TITLE) && xml.contains(DISPLAY_TITLE));
    [
        (
            xml.replacen(TITLE, "<TitleText>Yume &foo; Lullaby</TitleText>", 1),
            TEXT_ERROR,
        ),
        (
            xml.replacen(
                DISPLAY_TITLE,
                r#"<DisplayTitle ApplicableTerritoryCode="World&foo;wide" IsDefault="true">"#,
                1,
            ),
            ATTRIBUTE_ERROR,
        ),
    ]
}

#[test]
fn canonicalization_fails_on_an_undefined_entity() {
    let c14n = DB_C14N::new();
    for (xml, expected) in injected() {
        let error = c14n.canonicalize(&xml).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn canonicalization_still_decodes_predefined_entities_and_character_references() {
    let xml = sample().replacen(
        TITLE,
        "<TitleText>A &amp; B caf&#233; caf&#xE9;</TitleText>",
        1,
    );
    let out = DB_C14N::new().canonicalize(&xml).unwrap();
    assert!(
        out.contains("<TitleText>A &amp; B café café</TitleText>"),
        "{out}"
    );
}

#[test]
fn version_conversion_fails_on_an_undefined_entity() {
    let converter = VersionConverter::new();
    for (xml, expected) in injected() {
        match converter.convert(&xml, DdexVersion::Ern43, DdexVersion::Ern42, None) {
            ConverterResult::Failure { error, .. } => {
                assert!(error.contains(expected), "{error}")
            }
            ConverterResult::Success { .. } => panic!("conversion accepted {expected}"),
        }
    }
}
