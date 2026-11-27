//! Every public reader entry point applies the same policy: a DOCTYPE is rejected with the
//! `SecurityViolation` that `DDEXParser::parse` has always returned, and a reference to an
//! undefined entity is rejected naming the entity. One row per entry point, each against
//! the official sample `Samples43/1 Audio.xml` (a) as is, (b) with a DOCTYPE after the XML
//! declaration, (c) with `&foo;` in an attribute of the root start tag (so that even
//! `detect_version`, which stops at the root element, reads it).

use ddex_parser::parser::extension_capture::ExtensionAwareParser;
use ddex_parser::parser::mode::ParseMode;
use ddex_parser::parser::multi_release_parser::MultiReleaseParser;
use ddex_parser::parser::namespace_detector::NamespaceDetector;
use ddex_parser::parser::selective_parser::SelectiveParser;
use ddex_parser::parser::stream::StreamingParser;
use ddex_parser::parser::xpath_selector::XPathSelector;
use ddex_parser::parser::ParseOptions;
use ddex_parser::streaming::FastZeroCopyIterator;
use ddex_parser::transform::graph::GraphBuilder;
use ddex_parser::{DDEXParser, ERNVersion};
use std::io::Cursor;

const DOCTYPE_ERROR: &str = "DTD declarations are disabled for security";
const ENTITY_ERROR: &str = "undefined entity '&foo;' in an attribute value";
const ROOT: &str = "<ern:NewReleaseMessage";

fn sample() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Samples43/1 Audio.xml"
    );
    std::fs::read_to_string(path).expect("official sample examples/Samples43/1 Audio.xml")
}

fn with_doctype(xml: &str) -> String {
    let end = xml.find("?>").expect("an XML declaration") + 2;
    format!(
        "{}\n<!DOCTYPE ern:NewReleaseMessage>{}",
        &xml[..end],
        &xml[end..]
    )
}

fn with_undefined_entity(xml: &str) -> String {
    assert!(xml.contains(ROOT));
    xml.replacen(ROOT, &format!(r#"{ROOT} Test="&foo;""#), 1)
}

fn all<T, E: ToString>(items: impl IntoIterator<Item = Result<T, E>>) -> Result<(), String> {
    for item in items {
        item.map_err(|e| e.to_string())?;
    }
    Ok(())
}

type Reader = fn(&str) -> Result<(), String>;

fn readers() -> [(&'static str, Reader); 16] {
    [
        ("DDEXParser::parse", |xml| {
            DDEXParser::new()
                .parse(Cursor::new(xml.as_bytes()))
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        (
            "DDEXParser::parse_with_options (ParseMode::Stream)",
            |xml| {
                let options = ParseOptions {
                    mode: ParseMode::Stream,
                    ..ParseOptions::default()
                };
                DDEXParser::new()
                    .parse_with_options(Cursor::new(xml.as_bytes()), options)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            },
        ),
        ("DDEXParser::detect_version", |xml| {
            DDEXParser::new()
                .detect_version(Cursor::new(xml.as_bytes()))
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        ("DDEXParser::stream", |xml| {
            all(DDEXParser::new().stream(Cursor::new(xml.as_bytes())))
        }),
        ("DDEXParser::stream_parallel", |xml| {
            all(DDEXParser::new().stream_parallel(Cursor::new(xml.as_bytes())))
        }),
        ("FastZeroCopyIterator", |xml| {
            all(FastZeroCopyIterator::new(
                Cursor::new(xml.as_bytes()),
                ERNVersion::V4_3,
            ))
        }),
        ("StreamingParser::parse_header", |xml| {
            StreamingParser::new(Cursor::new(xml.as_bytes()), ERNVersion::V4_3)
                .parse_header()
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        ("SelectiveParser::extract_isrcs", |xml| {
            SelectiveParser::for_isrcs()
                .extract_isrcs(Cursor::new(xml.as_bytes()))
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        ("SelectiveParser::extract_isrcs_fast", |xml| {
            SelectiveParser::for_isrcs()
                .extract_isrcs_fast(Cursor::new(xml.as_bytes()))
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        ("XPathSelector::select", |xml| {
            XPathSelector::ddex_isrcs()
                .map_err(|e| e.to_string())?
                .select(Cursor::new(xml.as_bytes()))
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        ("XPathSelector::select_multiple", |xml| {
            XPathSelector::select_multiple(Cursor::new(xml.as_bytes()), &["//ISRC"])
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        ("MultiReleaseParser::count_releases", |xml| {
            MultiReleaseParser::new()
                .count_releases(Cursor::new(xml.as_bytes()))
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        ("MultiReleaseParser::parse_releases", |xml| {
            MultiReleaseParser::new()
                .parse_releases(Cursor::new(xml.as_bytes()))
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        ("ExtensionAwareParser::parse_with_extensions", |xml| {
            ExtensionAwareParser::new(true)
                .parse_with_extensions(xml)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        ("GraphBuilder::build_from_xml", |xml| {
            GraphBuilder::new(ERNVersion::V4_3)
                .build_from_xml(Cursor::new(xml.as_bytes()))
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
        ("NamespaceDetector::detect_from_xml", |xml| {
            NamespaceDetector::new()
                .detect_from_xml(Cursor::new(xml.as_bytes()))
                .map(|_| ())
                .map_err(|e| e.to_string())
        }),
    ]
}

#[test]
fn every_reader_accepts_the_official_sample() {
    let xml = sample();
    for (name, read) in readers() {
        if let Err(error) = read(&xml) {
            panic!("{name}: {error}");
        }
    }
}

#[test]
fn every_reader_rejects_a_doctype() {
    let xml = with_doctype(&sample());
    for (name, read) in readers() {
        match read(&xml) {
            Err(error) => assert!(error.contains(DOCTYPE_ERROR), "{name}: {error}"),
            Ok(()) => panic!("{name}: accepted a DOCTYPE"),
        }
    }
}

#[test]
fn every_reader_rejects_an_undefined_entity() {
    let xml = with_undefined_entity(&sample());
    for (name, read) in readers() {
        match read(&xml) {
            Err(error) => assert!(error.contains(ENTITY_ERROR), "{name}: {error}"),
            Ok(()) => panic!("{name}: accepted &foo;"),
        }
    }
}

#[test]
fn parse_keeps_its_doctype_error() {
    let xml = with_doctype(&sample());
    match DDEXParser::new().parse(Cursor::new(xml.as_bytes())) {
        Err(ddex_parser::error::ParseError::SecurityViolation { message }) => {
            assert_eq!(message, DOCTYPE_ERROR)
        }
        other => panic!("expected SecurityViolation, got {other:?}"),
    }
}
