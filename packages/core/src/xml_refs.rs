//! References to undefined entities.
//!
//! XML has five predefined entities (`&amp;`, `&lt;`, `&gt;`, `&apos;`, `&quot;`) and
//! character references (`&#233;`, `&#xE9;`). Any other `&name;` needs a declaration in a
//! DTD, and DDEX Suite does not read DTDs, so such a reference is an error. Before 0.5.1
//! some readers dropped it silently.
//!
//! [`ReferenceScanner`] finds such references in character data and in attribute values;
//! comments, CDATA sections, processing instructions and a DOCTYPE are skipped, since a `&`
//! there is not a reference. It takes the document in chunks of any size and reports the
//! line and column of the `&`. [`check_document`] scans a whole document, and
//! [`CheckedRead`] wraps a `BufRead` so that a streaming reader scans exactly the bytes it
//! consumes. Predefined entities and character references are left alone: whether a
//! character reference is valid stays the XML reader's decision.

use std::fmt;
use std::io::{self, BufRead, Read};

/// A reference to an entity that is neither predefined nor a character reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndefinedEntity {
    /// The entity name, between `&` and `;` (`foo` for `&foo;`).
    pub name: String,
    /// Whether the reference is in an attribute value (otherwise it is in character data).
    pub in_attribute: bool,
    /// 1-based line of the `&`.
    pub line: u64,
    /// 1-based column of the `&`, counted in characters.
    pub column: u64,
    /// Byte offset of the `&` from the start of the document.
    pub offset: u64,
}

impl fmt::Display for UndefinedEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "undefined entity '&{};'{} at line {}, column {}",
            self.name,
            if self.in_attribute {
                " in an attribute value"
            } else {
                ""
            },
            self.line,
            self.column
        )
    }
}

impl std::error::Error for UndefinedEntity {}

impl UndefinedEntity {
    /// The [`UndefinedEntity`] carried by an I/O error from [`CheckedRead`], if any.
    pub fn from_io_error(error: &io::Error) -> Option<&UndefinedEntity> {
        error.get_ref()?.downcast_ref::<UndefinedEntity>()
    }

    fn to_io_error(&self) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, self.clone())
    }
}

/// Whether `&name;` needs no DTD: one of the five predefined entities, or a character
/// reference (`#...`, whose validity is left to the XML reader).
pub fn is_defined(name: &[u8]) -> bool {
    name.first() == Some(&b'#') || matches!(name, b"amp" | b"lt" | b"gt" | b"apos" | b"quot")
}

/// Whether escaped text (character data or an attribute value as written) has a reference
/// to an undefined entity.
pub fn has_undefined(escaped: &[u8]) -> bool {
    let mut rest = escaped;
    while let Some(amp) = memchr::memchr(b'&', rest) {
        rest = &rest[amp + 1..];
        let end = rest
            .iter()
            .position(|&b| b == b';' || ends_name(b))
            .unwrap_or(rest.len());
        if end > 0 && end <= MAX_NAME && rest.get(end) == Some(&b';') && !is_defined(&rest[..end]) {
            return true;
        }
        rest = &rest[end..];
    }
    false
}

/// The first reference to an undefined entity in `xml`, a whole document.
pub fn check_document(xml: &[u8]) -> Result<(), UndefinedEntity> {
    let mut scanner = ReferenceScanner::new();
    scanner.feed(xml)
}

/// Every reference to an undefined entity in `xml`, a whole document, in document order.
pub fn undefined_entities(xml: &[u8]) -> Vec<UndefinedEntity> {
    let mut scanner = ReferenceScanner::new();
    let mut found = Vec::new();
    scanner.scan(xml, &mut |entity| {
        found.push(entity);
        true
    });
    found
}

/// 1-based line and column (in characters) of byte `offset` in `text`.
pub fn line_column(text: &[u8], offset: usize) -> (u64, u64) {
    let position = Position::START.advance(&text[..offset.min(text.len())]);
    (position.line, position.column)
}

/// Longest entity name looked at; a longer run is not a reference this scanner reports.
const MAX_NAME: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    offset: u64,
    line: u64,
    column: u64,
}

impl Position {
    const START: Position = Position {
        offset: 0,
        line: 1,
        column: 1,
    };

    fn advance(mut self, bytes: &[u8]) -> Position {
        self.offset += bytes.len() as u64;
        match memchr::memrchr(b'\n', bytes) {
            Some(last) => {
                self.line += bytes.iter().filter(|&&b| b == b'\n').count() as u64;
                self.column = 1 + chars(&bytes[last + 1..]);
            }
            None => self.column += chars(bytes),
        }
        self
    }
}

/// UTF-8 characters in `bytes` (bytes that are not continuation bytes).
fn chars(bytes: &[u8]) -> u64 {
    bytes.iter().filter(|&&b| b & 0xC0 != 0x80).count() as u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Character data.
    Text,
    /// After `<`.
    Lt,
    /// After `<!`.
    Bang,
    /// After `<!-`.
    BangDash,
    /// After `<!` and this many bytes of `[CDATA[`.
    CdataOpen(u8),
    /// Inside a start or end tag, outside attribute values.
    Tag,
    /// Inside an attribute value delimited by this quote.
    Quoted(u8),
    /// Inside a comment, until `-->`.
    Comment,
    /// Inside a CDATA section, until `]]>`.
    Cdata,
    /// Inside a processing instruction or the XML declaration, until `?>`.
    Pi,
    /// Inside a DOCTYPE or another `<!` declaration, until its `>` (internal subset
    /// brackets are counted).
    Doctype,
    /// After `&`, reading the entity name.
    Ref,
}

const CDATA_OPEN: &[u8; 7] = b"[CDATA[";

/// Finds references to undefined entities in a document fed in chunks (see the module
/// docs). After the first one, [`ReferenceScanner::feed`] keeps returning it.
#[derive(Debug, Clone)]
pub struct ReferenceScanner {
    state: State,
    /// State to return to when the reference ends (`Text` or `Quoted`).
    ref_return: State,
    name: Vec<u8>,
    /// Position of the `&` of a reference that started in an earlier chunk.
    ref_position: Position,
    /// Position of the next byte.
    position: Position,
    /// The last two bytes fed, for terminators split across chunks.
    previous: [u8; 2],
    doctype_depth: usize,
    error: Option<UndefinedEntity>,
}

impl Default for ReferenceScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl ReferenceScanner {
    pub fn new() -> Self {
        Self {
            state: State::Text,
            ref_return: State::Text,
            name: Vec::new(),
            ref_position: Position::START,
            position: Position::START,
            previous: [0; 2],
            doctype_depth: 0,
            error: None,
        }
    }

    /// The first undefined reference found so far.
    pub fn error(&self) -> Option<&UndefinedEntity> {
        self.error.as_ref()
    }

    /// Scans the next chunk of the document.
    pub fn feed(&mut self, chunk: &[u8]) -> Result<(), UndefinedEntity> {
        if self.error.is_none() {
            let mut first = None;
            self.scan(chunk, &mut |entity| {
                first = Some(entity);
                false
            });
            self.error = first;
        }
        match &self.error {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }

    /// Scans `chunk`, calling `found` for each undefined reference until it returns false.
    fn scan(&mut self, chunk: &[u8], found: &mut dyn FnMut(UndefinedEntity) -> bool) {
        let start = self.position;
        // Index of the `&` when the current reference started in this chunk.
        let mut ref_in_chunk: Option<usize> = None;
        let mut i = 0;
        while i < chunk.len() {
            match self.state {
                State::Text => match memchr::memchr2(b'&', b'<', &chunk[i..]) {
                    None => i = chunk.len(),
                    Some(k) => {
                        let j = i + k;
                        if chunk[j] == b'&' {
                            self.begin_ref(State::Text);
                            ref_in_chunk = Some(j);
                        } else {
                            self.state = State::Lt;
                        }
                        i = j + 1;
                    }
                },
                State::Lt => {
                    // The byte after `<` is looked at again in the new state, except `!`/`?`.
                    match chunk[i] {
                        b'!' => {
                            self.state = State::Bang;
                            i += 1;
                        }
                        b'?' => {
                            self.state = State::Pi;
                            i += 1;
                        }
                        _ => self.state = State::Tag,
                    }
                }
                State::Bang => match chunk[i] {
                    b'-' => {
                        self.state = State::BangDash;
                        i += 1;
                    }
                    b'[' => {
                        self.state = State::CdataOpen(1);
                        i += 1;
                    }
                    _ => self.begin_doctype(),
                },
                State::BangDash => {
                    if chunk[i] == b'-' {
                        self.state = State::Comment;
                        i += 1;
                    } else {
                        self.begin_doctype();
                    }
                }
                State::CdataOpen(n) => {
                    if chunk[i] == CDATA_OPEN[n as usize] {
                        i += 1;
                        self.state = if n as usize + 1 == CDATA_OPEN.len() {
                            State::Cdata
                        } else {
                            State::CdataOpen(n + 1)
                        };
                    } else {
                        self.begin_doctype();
                    }
                }
                State::Tag => match memchr::memchr3(b'"', b'\'', b'>', &chunk[i..]) {
                    None => i = chunk.len(),
                    Some(k) => {
                        let j = i + k;
                        self.state = match chunk[j] {
                            b'>' => State::Text,
                            quote => State::Quoted(quote),
                        };
                        i = j + 1;
                    }
                },
                State::Quoted(quote) => match memchr::memchr2(quote, b'&', &chunk[i..]) {
                    None => i = chunk.len(),
                    Some(k) => {
                        let j = i + k;
                        if chunk[j] == b'&' {
                            self.begin_ref(State::Quoted(quote));
                            ref_in_chunk = Some(j);
                        } else {
                            self.state = State::Tag;
                        }
                        i = j + 1;
                    }
                },
                State::Comment | State::Cdata | State::Pi => {
                    match memchr::memchr(b'>', &chunk[i..]) {
                        None => i = chunk.len(),
                        Some(k) => {
                            let j = i + k;
                            let closed = match self.state {
                                State::Comment => {
                                    self.before(chunk, j, 1) == b'-'
                                        && self.before(chunk, j, 2) == b'-'
                                }
                                State::Cdata => {
                                    self.before(chunk, j, 1) == b']'
                                        && self.before(chunk, j, 2) == b']'
                                }
                                _ => self.before(chunk, j, 1) == b'?',
                            };
                            if closed {
                                self.state = State::Text;
                            }
                            i = j + 1;
                        }
                    }
                }
                State::Doctype => match memchr::memchr3(b'[', b']', b'>', &chunk[i..]) {
                    None => i = chunk.len(),
                    Some(k) => {
                        let j = i + k;
                        match chunk[j] {
                            b'[' => self.doctype_depth += 1,
                            b']' => self.doctype_depth = self.doctype_depth.saturating_sub(1),
                            _ if self.doctype_depth == 0 => self.state = State::Text,
                            _ => {}
                        }
                        i = j + 1;
                    }
                },
                State::Ref => {
                    let byte = chunk[i];
                    if byte == b';' {
                        i += 1;
                        self.state = self.ref_return;
                        if !self.name.is_empty() && !is_defined(&self.name) {
                            let at = match ref_in_chunk {
                                Some(j) => start.advance(&chunk[..j]),
                                None => self.ref_position,
                            };
                            let entity = UndefinedEntity {
                                name: String::from_utf8_lossy(&self.name).into_owned(),
                                in_attribute: matches!(self.ref_return, State::Quoted(_)),
                                line: at.line,
                                column: at.column,
                                offset: at.offset,
                            };
                            if !found(entity) {
                                return;
                            }
                        }
                    } else if ends_name(byte) || self.name.len() >= MAX_NAME {
                        // Not a reference this scanner reports (the XML reader judges it);
                        // the byte is looked at again in the state the reference started in.
                        self.state = self.ref_return;
                    } else {
                        self.name.push(byte);
                        i += 1;
                    }
                }
            }
        }

        if self.state == State::Ref {
            if let Some(j) = ref_in_chunk {
                self.ref_position = start.advance(&chunk[..j]);
            }
        }
        self.position = start.advance(chunk);
        match chunk.len() {
            0 => {}
            1 => self.previous = [self.previous[1], chunk[0]],
            n => self.previous = [chunk[n - 2], chunk[n - 1]],
        }
    }

    fn begin_ref(&mut self, return_to: State) {
        self.state = State::Ref;
        self.ref_return = return_to;
        self.name.clear();
    }

    fn begin_doctype(&mut self) {
        self.state = State::Doctype;
        self.doctype_depth = 0;
    }

    /// The byte `back` positions before `chunk[j]`, looking into earlier chunks (`back` <= 2).
    fn before(&self, chunk: &[u8], j: usize, back: usize) -> u8 {
        if j >= back {
            chunk[j - back]
        } else {
            self.previous[2 - (back - j)]
        }
    }
}

/// Bytes that cannot be part of an entity name (the reference is malformed, not undefined).
fn ends_name(byte: u8) -> bool {
    matches!(
        byte,
        b' ' | b'\t' | b'\n' | b'\r' | b'<' | b'>' | b'&' | b'"' | b'\''
    )
}

/// A `BufRead` that scans the bytes its reader consumes with a [`ReferenceScanner`].
///
/// Once an undefined reference has been consumed, [`CheckedRead::error`] returns it and
/// every further read fails with an `InvalidData` I/O error carrying it
/// ([`UndefinedEntity::from_io_error`]). A reader that checks `error()` after each event
/// fails on the event that holds the reference.
#[derive(Debug)]
pub struct CheckedRead<R> {
    inner: R,
    scanner: ReferenceScanner,
}

impl<R> CheckedRead<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            scanner: ReferenceScanner::new(),
        }
    }

    /// The first undefined reference consumed so far.
    pub fn error(&self) -> Option<&UndefinedEntity> {
        self.scanner.error()
    }

    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read> Read for CheckedRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(error) = self.scanner.error() {
            return Err(error.to_io_error());
        }
        let n = self.inner.read(buf)?;
        self.scanner
            .feed(&buf[..n])
            .map_err(|error| error.to_io_error())?;
        Ok(n)
    }
}

impl<R: BufRead> BufRead for CheckedRead<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if let Some(error) = self.scanner.error() {
            return Err(error.to_io_error());
        }
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        if amt > 0 {
            // `consume` follows a `fill_buf` that returned at least `amt` bytes, so this
            // returns the same buffer without reading.
            if let Ok(buf) = self.inner.fill_buf() {
                let n = amt.min(buf.len());
                // An error is kept by the scanner and reported by the next read.
                let _ = self.scanner.feed(&buf[..n]);
            }
        }
        self.inner.consume(amt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first(xml: &str) -> Option<UndefinedEntity> {
        check_document(xml.as_bytes()).err()
    }

    #[test]
    fn predefined_entities_and_character_references_are_defined() {
        let xml = r#"<a b="&amp;&lt;&gt;&apos;&quot;&#233;&#xE9;">&amp; caf&#233; caf&#xE9; &lt;&gt;&apos;&quot;</a>"#;
        assert_eq!(first(xml), None);
    }

    #[test]
    fn undefined_entity_in_text() {
        let error = first("<a>\n  <b>x &foo; y</b>\n</a>").unwrap();
        assert_eq!(error.name, "foo");
        assert!(!error.in_attribute);
        assert_eq!((error.line, error.column, error.offset), (2, 8, 11));
        assert_eq!(
            error.to_string(),
            "undefined entity '&foo;' at line 2, column 8"
        );
    }

    #[test]
    fn undefined_entity_in_attribute() {
        let error = first("<a>\n<b c='1' d=\"x&foo;\"/></a>").unwrap();
        assert_eq!(error.name, "foo");
        assert!(error.in_attribute);
        assert_eq!((error.line, error.column), (2, 14));
        assert_eq!(
            error.to_string(),
            "undefined entity '&foo;' in an attribute value at line 2, column 14"
        );
    }

    #[test]
    fn column_counts_characters() {
        // "夢" is three bytes in UTF-8 and one column.
        let error = first("<a>夢&foo;</a>").unwrap();
        assert_eq!((error.line, error.column, error.offset), (1, 5, 6));
    }

    #[test]
    fn markup_that_is_not_character_data_is_skipped() {
        let xml = "<?xml version=\"1.0\"?><!-- &c; --><a><![CDATA[&d; ]] > ]]><?pi &e; ?>\
                   <b x='>' y=\"'&amp;\">&gt;</b></a>";
        assert_eq!(first(xml), None);
        // After each skipped construct, references count again.
        assert_eq!(first("<!-- -- > --><a>&x;</a>").unwrap().name, "x");
        assert_eq!(first("<a><![CDATA[ ] ]> ]]>&y;</a>").unwrap().name, "y");
    }

    #[test]
    fn doctype_internal_subset_is_skipped() {
        let xml = "<!DOCTYPE a [<!ENTITY e \"&#38;x;\"> <!ENTITY f 'g'>]><a>&amp;</a>";
        assert_eq!(first(xml), None);
        assert_eq!(first("<!DOCTYPE a [ ]><a>&z;</a>").unwrap().name, "z");
    }

    #[test]
    fn malformed_references_are_left_to_the_xml_reader() {
        assert_eq!(first("<a>& foo; &; a&b</a>"), None);
        assert_eq!(first("<a b='&x y;'/>"), None);
    }

    #[test]
    fn every_chunk_boundary_gives_the_same_result() {
        let xml = "<?xml version=\"1.0\"?>\n<!-- &c; -->\n<a k=\"&amp;\">\
                   <![CDATA[&d;]]>夢\n  <b c=\"v\">t &undefined; u</b></a>";
        let expected = first(xml).unwrap();
        assert_eq!(expected.name, "undefined");
        assert_eq!(expected.line, 4);
        let bytes = xml.as_bytes();
        for size in 1..=bytes.len() {
            let mut scanner = ReferenceScanner::new();
            let result = bytes.chunks(size).try_for_each(|chunk| scanner.feed(chunk));
            assert_eq!(result.err().as_ref(), Some(&expected), "chunk size {size}");
        }
    }

    #[test]
    fn all_undefined_entities_are_listed() {
        let found = undefined_entities(b"<a b='&x;'>&y;&amp;&z;</a>");
        let names: Vec<_> = found.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["x", "y", "z"]);
        assert!(found[0].in_attribute && !found[1].in_attribute);
    }

    #[test]
    fn escaped_values_with_undefined_entities() {
        assert!(has_undefined(b"World&foo;wide"));
        assert!(has_undefined(b"&amp;&bar;"));
        assert!(!has_undefined(b"&amp; &#233; &#xE9; & x;"));
    }

    #[test]
    fn line_column_of_offset() {
        assert_eq!(line_column(b"ab\ncd", 0), (1, 1));
        assert_eq!(line_column(b"ab\ncd", 4), (2, 2));
    }

    #[test]
    fn checked_read_fails_once_the_reference_is_consumed() {
        let xml = b"<a>ok</a><b>&foo;</b>".as_slice();
        let mut input = CheckedRead::new(xml);
        let mut out = String::new();
        let error = input.read_to_string(&mut out).unwrap_err();
        let entity = UndefinedEntity::from_io_error(&error).unwrap();
        assert_eq!((entity.name.as_str(), entity.column), ("foo", 13));
        assert_eq!(input.error(), Some(entity));

        let mut input = CheckedRead::new(b"<a>x</a>".as_slice());
        let buf = input.fill_buf().unwrap().len();
        input.consume(buf);
        assert!(input.error().is_none());
        assert!(input.fill_buf().unwrap().is_empty());
    }
}
