//! Text and attribute helpers for quick-xml 0.41.
//!
//! Since quick-xml 0.38 every entity or character reference in character data (`&amp;`,
//! `&#13;`, ...) is its own `Event::GeneralRef`, so `a &amp; b` arrives as `Text("a ")`,
//! `GeneralRef("amp")`, `Text(" b")`. Before, it was one escaped `Text` event. [`TextRun`]
//! joins such a run back into that one escaped text, so code written against the old events
//! keeps its behaviour. [`attr_value`] resolves references in an attribute value without the
//! attribute-value normalization that quick-xml 0.40 added to `unescape_value`, as before.
//!
//! References to undefined entities (`&foo;` with no DTD) and a DOCTYPE are errors in every
//! reader: readers over a `BufRead` read through [`checked`] and call [`check`] after each
//! event, and readers that have the whole document call [`check_document`] first. They
//! report [`undefined_entity`] (which names the entity and gives its line and column) and
//! [`doctype_error`].

use crate::error::ParseError;
use ddex_core::xml_refs::{CheckedRead, Doctype, ReferenceScanner, UndefinedEntity};
use quick_xml::escape::{unescape, EscapeError};
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesRef, Event};

/// Collects a run of `Text` and `GeneralRef` events as the escaped bytes that quick-xml
/// < 0.38 reported in a single `Text` event.
#[derive(Debug, Default)]
pub(crate) struct TextRun {
    raw: Vec<u8>,
    active: bool,
}

impl TextRun {
    /// Adds the event to the run if it is `Text` or `GeneralRef`; returns whether it was.
    pub(crate) fn push(&mut self, event: &Event) -> bool {
        match event {
            Event::Text(text) => {
                self.raw.extend_from_slice(text);
                self.active = true;
                true
            }
            Event::GeneralRef(reference) => {
                self.raw.push(b'&');
                self.raw.extend_from_slice(reference);
                self.raw.push(b';');
                self.active = true;
                true
            }
            _ => false,
        }
    }

    /// The escaped bytes of the finished run, or `None` when no text was collected.
    pub(crate) fn take(&mut self) -> Option<Vec<u8>> {
        if !self.active {
            return None;
        }
        self.active = false;
        Some(std::mem::take(&mut self.raw))
    }
}

/// The text of one reference (`amp` -> `&`, `#13` -> carriage return). An undefined entity
/// yields an empty string, but the readers have failed on it before (see the module docs).
pub(crate) fn ref_text(reference: &BytesRef) -> String {
    let raw = format!("&{};", String::from_utf8_lossy(reference));
    unescape(&raw).map(|s| s.into_owned()).unwrap_or_default()
}

/// An attribute value with its references resolved and nothing else changed (the
/// behaviour of `Attribute::unescape_value` before quick-xml 0.40: invalid UTF-8 and
/// unknown references are errors).
pub(crate) fn attr_value(attr: &Attribute) -> Result<String, String> {
    let text = std::str::from_utf8(&attr.value).map_err(|e| e.to_string())?;
    unescape(text)
        .map(|s| s.into_owned())
        .map_err(|e: EscapeError| e.to_string())
}

/// An input that is scanned for references to undefined entities and for a DOCTYPE as it
/// is read.
pub(crate) type Checked<R> = CheckedRead<R>;

/// `inner` read through a [`Checked`] input that rejects a DOCTYPE.
pub(crate) fn checked<R>(inner: R) -> Checked<R> {
    CheckedRead::rejecting_doctype(inner)
}

/// The error for a DOCTYPE, on every reader: the `SecurityViolation` that
/// `DDEXParser::parse` has always returned for one.
pub(crate) fn doctype_error(_doctype: &Doctype) -> ParseError {
    ParseError::SecurityViolation {
        message: "DTD declarations are disabled for security".to_string(),
    }
}

/// A quick-xml read error, with the errors of a [`Checked`] input (an undefined entity, a
/// DOCTYPE) reported as such.
pub(crate) fn read_error(error: quick_xml::Error) -> ParseError {
    if let quick_xml::Error::Io(io) = &error {
        if let Some(entity) = UndefinedEntity::from_io_error(io) {
            return undefined_entity(entity);
        }
        if let Some(doctype) = Doctype::from_io_error(io) {
            return doctype_error(doctype);
        }
    }
    ParseError::XmlError(format!("XML parsing error: {}", error))
}

/// The error for a reference to an undefined entity: `MalformedXml` at the byte offset of
/// the `&`, with a message such as `undefined entity '&foo;' at line 74, column 30`.
pub(crate) fn undefined_entity(entity: &UndefinedEntity) -> ParseError {
    ParseError::MalformedXml {
        message: entity.to_string(),
        position: entity.offset as usize,
    }
}

/// Fails if the input has consumed a reference to an undefined entity or a DOCTYPE. Called
/// after each event, it fails on the event that holds it.
pub(crate) fn check<R>(input: &Checked<R>) -> Result<(), ParseError> {
    if let Some(entity) = input.error() {
        return Err(undefined_entity(entity));
    }
    match input.doctype() {
        Some(doctype) => Err(doctype_error(doctype)),
        None => Ok(()),
    }
}

/// Fails on the first reference to an undefined entity or DOCTYPE in a whole document,
/// whichever comes first.
pub(crate) fn check_document(xml: &[u8]) -> Result<(), ParseError> {
    let mut scanner = ReferenceScanner::rejecting_doctype();
    scanner
        .feed(xml)
        .map_err(|entity| undefined_entity(&entity))?;
    match scanner.doctype() {
        Some(doctype) => Err(doctype_error(doctype)),
        None => Ok(()),
    }
}
