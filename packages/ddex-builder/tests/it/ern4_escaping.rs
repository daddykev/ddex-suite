//! Escaping property test: titles and artists made of quotes, apostrophes, `<script>`,
//! `&`, `]]>`, emoji, RTL text and control characters. Either the build fails with an
//! error naming the field (characters XML 1.0 does not allow), or the output is
//! well-formed, schema-valid, and ddex-parser reads every text back unchanged.

use crate::ern4_support::{assert_xsd_valid, minimal_single, parse};
use ddex_builder::{BuildError, BuildOptions, DDEXBuilder};
use proptest::prelude::*;

const FRAGMENTS: &[&str] = &[
    "Title",
    "a",
    " ",
    "  ",
    "\"",
    "'",
    "\"quoted\" it's",
    "<script>alert('x')</script>",
    "<b>bold</b>",
    "&",
    "&amp;",
    "&lt;",
    "<",
    ">",
    "]]>",
    "<![CDATA[x]]>",
    "<!-- c -->",
    "🎵",
    "😀👍🏽",
    "مرحبا بالعالم",
    "שלום",
    "\u{202E}",
    "\t",
    "\n",
    "\r",
    "\r\n",
    "\u{0}",
    "\u{1}",
    "\u{8}",
    "\u{B}",
    "\u{C}",
    "\u{1F}",
    "\u{FFFE}",
    "\u{FFFF}",
];

fn text() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(FRAGMENTS), 1..6)
        // ddex-parser keeps text exactly as written, surrounding whitespace included.
        .prop_map(|parts| parts.concat())
}

fn has_illegal(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(c, '\u{0}'..='\u{8}' | '\u{B}' | '\u{C}' | '\u{E}'..='\u{1F}' | '\u{FFFE}' | '\u{FFFF}')
    })
}

/// Parse the whole document; panics if it is not well-formed.
fn assert_well_formed(xml: &str) {
    let mut reader = quick_xml::Reader::from_str(xml);
    reader.config_mut().check_end_names = true;
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {}
            Err(e) => panic!("not well-formed: {e}\n{xml}"),
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 96, ..ProptestConfig::default() })]

    #[test]
    fn titles_and_artists_are_escaped_or_rejected(title in text(), artist in text(), v43 in any::<bool>()) {
        let version = if v43 { "4.3" } else { "4.2" };
        let mut request = minimal_single(version);
        let release = &mut request.releases[0];
        release.title[0].text = title.clone();
        release.artist = Some(artist.clone());
        release.tracks[0].title = title.clone();
        release.tracks[0].artist = Some(artist.clone());

        match DDEXBuilder::new().build(request, BuildOptions::default()) {
            Err(BuildError::InvalidFormat { field, message }) => {
                prop_assert!(has_illegal(&title) || has_illegal(&artist), "rejected legal text: {field}: {message}");
                prop_assert!(field.starts_with("releases[0]."), "field not named: {field}");
                prop_assert!(message.contains("XML 1.0 does not allow"), "{message}");
            }
            Err(e) => prop_assert!(false, "unexpected error: {e}"),
            Ok(result) => {
                prop_assert!(!has_illegal(&title) && !has_illegal(&artist), "illegal character accepted");
                let xml = result.xml;
                assert_well_formed(&xml);
                prop_assert!(!xml.contains("<script>"), "raw <script> element in output");
                assert_xsd_valid(&xml, version, "escaping property");
                let flat = parse(&xml).flat;
                let release = &flat.releases[0];
                prop_assert_eq!(release.default_title.as_deref(), Some(title.as_str()));
                prop_assert_eq!(release.display_artist.as_deref(), Some(artist.as_str()));
                prop_assert_eq!(release.tracks[0].title.as_deref(), Some(title.as_str()));
                prop_assert_eq!(release.tracks[0].display_artist.as_deref(), Some(artist.as_str()));
            }
        }
    }
}

/// Each named case on its own, so a regression points at the exact input.
#[test]
fn named_cases() {
    let legal = [
        "Say \"hi\"",
        "It's",
        "<script>alert('XSS')</script>",
        "Rock & Roll",
        "]]>",
        "🎵 Song 😀",
        "مرحبا بالعالم",
        "line one\nline two",
        "tab\there",
        "cr\rhere",
    ];
    for text in legal {
        let mut request = minimal_single("4.3");
        request.releases[0].title[0].text = text.to_string();
        request.releases[0].artist = Some(text.to_string());
        let xml = DDEXBuilder::new()
            .build(request, BuildOptions::default())
            .unwrap_or_else(|e| panic!("{text:?}: {e}"))
            .xml;
        assert_well_formed(&xml);
        assert_xsd_valid(&xml, "4.3", text);
        let flat = parse(&xml).flat;
        assert_eq!(
            flat.releases[0].default_title.as_deref(),
            Some(text),
            "{xml}"
        );
        assert_eq!(
            flat.releases[0].display_artist.as_deref(),
            Some(text),
            "{xml}"
        );
    }

    for (text, code) in [
        ("bell\u{7}", "U+0007"),
        ("nul\u{0}", "U+0000"),
        ("bad\u{FFFE}", "U+FFFE"),
    ] {
        let mut request = minimal_single("4.3");
        request.releases[0].tracks[0].title = text.to_string();
        let err = DDEXBuilder::new()
            .build(request, BuildOptions::default())
            .expect_err("illegal character must be rejected");
        let message = err.to_string();
        assert!(message.contains("releases[0].tracks[0].title"), "{message}");
        assert!(message.contains(code), "{message}");
    }
}

/// Tab, LF and CR survive build -> parse: in an attribute value they are written as
/// `&#9;`, `&#10;`, `&#13;` (XML 1.0 attribute-value normalization would turn literal ones
/// into spaces), and in text a CR is written as `&#13;` (a literal one would become a LF).
#[test]
fn tab_lf_and_cr_survive_a_round_trip_in_attributes_and_text() {
    let text = "a\tb\nc\rd\r\ne";
    let language = "x\ty\nz\rw";
    for version in ["4.3", "4.2"] {
        let mut request = minimal_single(version);
        request.releases[0].title[0].text = text.to_string();
        request.releases[0].title[0].language_code = Some(language.to_string());
        let xml = DDEXBuilder::new()
            .build(request, BuildOptions::default())
            .unwrap_or_else(|e| panic!("{version}: {e}"))
            .xml;
        assert!(xml.contains("x&#9;y&#10;z&#13;w"), "{version}: {xml}");
        assert!(xml.contains("c&#13;d&#13;\ne"), "{version}: {xml}");
        let parsed = parse(&xml);
        let title = &parsed.graph.releases[0].display_title_text[0];
        assert_eq!(title.text, text, "{version}");
        assert_eq!(
            title.language_and_script_code.as_deref(),
            Some(language),
            "{version}"
        );
    }
}
