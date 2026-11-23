//! Text and attribute helpers for quick-xml 0.41.
//!
//! Since quick-xml 0.38 every entity or character reference in character data (`&amp;`,
//! `&#13;`, ...) is its own `Event::GeneralRef`, so `a &amp; b` arrives as `Text("a ")`,
//! `GeneralRef("amp")`, `Text(" b")`. Before, it was one escaped `Text` event. [`TextRun`]
//! joins such a run back into that one escaped text, so code written against the old events
//! keeps its behaviour. [`attr_value`] resolves references in an attribute value without the
//! attribute-value normalization that quick-xml 0.40 added to `unescape_value`, as before.

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

/// The text of one reference (`amp` -> `&`, `#13` -> carriage return). An unknown entity
/// (DTDs are rejected, so only the predefined ones exist) yields an empty string.
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
