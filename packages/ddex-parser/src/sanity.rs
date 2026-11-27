//! The checks behind `DDEXParser::sanity_check`: ERN version detection (the parser's own
//! [`VersionDetector`]) and an XML well-formedness pass. Not XSD or AVS validation.

use crate::parser::detector::VersionDetector;
use ddex_core::models::versions::ERNVersion;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::io::Cursor;

/// Stop collecting after this many problems (one syntax error ends the pass anyway).
const MAX_ERRORS: usize = 20;

/// Result of the checks. `version` is `None` when no ERN namespace was found.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SanityReport {
    pub version: Option<ERNVersion>,
    pub errors: Vec<String>,
}

pub(crate) fn check(bytes: &[u8]) -> SanityReport {
    let mut errors = Vec::new();
    if bytes.iter().all(u8::is_ascii_whitespace) {
        errors.push("Empty input".to_string());
        return SanityReport {
            version: None,
            errors,
        };
    }

    let version = match VersionDetector::detect(Cursor::new(bytes)) {
        Ok(version) => Some(version),
        Err(e) => {
            errors.push(format!("ERN version not detected: {e}"));
            None
        }
    };
    errors.extend(well_formedness_errors(bytes));
    errors.truncate(MAX_ERRORS);

    SanityReport { version, errors }
}

/// XML 1.0 well-formedness problems, in document order: syntax errors (including
/// mismatched end tags), duplicate or malformed attributes, references to undefined
/// entities (there is no DTD; the parser rejects DTDs), unclosed elements, a missing or
/// second root element, and text outside the root.
fn well_formedness_errors(bytes: &[u8]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().check_end_names = true;
    let mut buf = Vec::new();
    let mut depth = 0usize;
    let mut roots = 0usize;

    loop {
        if errors.len() >= MAX_ERRORS {
            break;
        }
        let position = reader.buffer_position();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                check_root(depth, &mut roots, position, &mut errors);
                let end = reader.buffer_position();
                check_attributes(e, bytes, position, end, &mut errors);
                depth += 1;
            }
            Ok(Event::Empty(ref e)) => {
                check_root(depth, &mut roots, position, &mut errors);
                let end = reader.buffer_position();
                check_attributes(e, bytes, position, end, &mut errors);
            }
            Ok(Event::End(_)) => depth = depth.saturating_sub(1),
            Ok(Event::Text(ref t)) => {
                if depth == 0 && !t.iter().all(u8::is_ascii_whitespace) {
                    errors.push(format!("Text outside the root element at byte {position}"));
                }
            }
            Ok(Event::GeneralRef(ref r)) => {
                let raw = format!("&{};", String::from_utf8_lossy(r));
                if !ddex_core::xml_refs::is_defined(r) {
                    let (line, column) = ddex_core::xml_refs::line_column(bytes, position as usize);
                    errors.push(format!(
                        "Undefined entity '{raw}' at line {line}, column {column}"
                    ));
                } else if quick_xml::escape::unescape(&raw).is_err() {
                    errors.push(format!("Undefined entity {raw} at byte {position}"));
                } else if depth == 0 {
                    errors.push(format!("Text outside the root element at byte {position}"));
                }
            }
            Ok(Event::CData(_)) if depth == 0 => {
                errors.push(format!("CDATA outside the root element at byte {position}"));
            }
            Ok(Event::DocType(_)) => {
                errors.push(format!(
                    "DOCTYPE at byte {position}: DTDs are not accepted (the parser rejects them)"
                ));
            }
            Ok(Event::Eof) => {
                if depth > 0 {
                    errors.push(format!(
                        "{depth} element(s) not closed at the end of the input"
                    ));
                }
                if roots == 0 {
                    errors.push("No root element".to_string());
                }
                break;
            }
            Ok(_) => {}
            Err(e) => {
                errors.push(format!(
                    "Not well-formed XML at byte {}: {e}",
                    reader.error_position()
                ));
                break;
            }
        }
        buf.clear();
    }
    errors
}

fn check_root(depth: usize, roots: &mut usize, position: u64, errors: &mut Vec<String>) {
    if depth == 0 {
        *roots += 1;
        if *roots == 2 {
            errors.push(format!("Second root element at byte {position}"));
        }
    }
}

/// The tag `e` spans bytes `position..end` of the document `bytes`; a reference to an
/// undefined entity in one of its attribute values is reported with its line and column.
fn check_attributes(
    e: &BytesStart,
    bytes: &[u8],
    position: u64,
    end: u64,
    errors: &mut Vec<String>,
) {
    let tag = &bytes[position as usize..end as usize];
    for entity in ddex_core::xml_refs::undefined_entities(tag) {
        let (line, column) =
            ddex_core::xml_refs::line_column(bytes, (position + entity.offset) as usize);
        errors.push(format!(
            "Undefined entity '&{};' in an attribute value at line {line}, column {column}",
            entity.name
        ));
    }
    for attr in e.attributes() {
        match attr {
            Ok(attr) => {
                let valid = std::str::from_utf8(&attr.value)
                    .ok()
                    .is_some_and(|value| quick_xml::escape::unescape(value).is_ok());
                // An undefined entity is reported above.
                if !valid && !ddex_core::xml_refs::has_undefined(&attr.value) {
                    errors.push(format!(
                        "Invalid value of attribute {} at byte {position}",
                        String::from_utf8_lossy(attr.key.as_ref())
                    ));
                }
            }
            Err(err) => {
                errors.push(format!("Invalid attribute at byte {position}: {err}"));
                return;
            }
        }
    }
}
