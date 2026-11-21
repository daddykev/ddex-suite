// src/streaming/comprehensive.rs
//! Comprehensive streaming DDEX parser using model-aligned types

#[allow(dead_code)] // Experimental streaming parser implementation
use crate::error::ParseError;
use ddex_core::models::streaming_types::*;
use ddex_core::models::LocalizedString;
use ddex_core::models::{graph::*, versions::ERNVersion};
use log::error;
use quick_xml::{events::Event, Reader};
use std::collections::HashMap;
use std::io::BufRead;
use std::time::Instant;

/// Comprehensive streaming element types using model-aligned builders
#[derive(Debug, Clone)]
pub enum StreamingElement {
    Header(Box<MessageHeader>),
    Release(Release),
    Resource(Resource),
    Party(Party),
    EndOfStream,
}

/// Parser state using streaming builders
#[derive(Debug, Clone)]
enum ParserState {
    Initial,
    InHeader(Box<MessageHeaderBuilder>),
    InRelease(Box<ReleaseBuilder>),
    InResource(Box<ResourceBuilder>),
    InParty(Box<PartyBuilder>),
    Complete,
}

/// Comprehensive streaming parser
pub struct ComprehensiveStreamingParser<R: BufRead> {
    reader: Reader<R>,
    buffer: Vec<u8>,
    state: ParserState,
    current_path: Vec<String>,
    current_depth: usize,
    text_buffer: String,
    attributes: HashMap<String, String>,
    bytes_processed: u64,
    elements_yielded: usize,
    start_time: Instant,
}

impl<R: BufRead> ComprehensiveStreamingParser<R> {
    pub fn new(reader: R, _version: ERNVersion) -> Self {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        xml_reader.config_mut().check_end_names = true;

        Self {
            reader: xml_reader,
            buffer: Vec::with_capacity(8192),
            state: ParserState::Initial,
            current_path: Vec::new(),
            current_depth: 0,
            text_buffer: String::new(),
            attributes: HashMap::new(),
            bytes_processed: 0,
            elements_yielded: 0,
            start_time: Instant::now(),
        }
    }

    pub fn parse_next(&mut self) -> Result<Option<StreamingElement>, ParseError> {
        loop {
            self.buffer.clear();
            let event = self.reader.read_event_into(&mut self.buffer);
            let bytes_position = self.reader.buffer_position();

            match event {
                Ok(Event::Start(e)) => {
                    let name_bytes = e.name();
                    let name = std::str::from_utf8(name_bytes.as_ref())?;
                    self.current_path.push(name.to_string());
                    self.current_depth += 1;

                    // Extract attributes
                    self.attributes.clear();
                    for attr in e.attributes() {
                        let attr = attr?;
                        let key = std::str::from_utf8(attr.key.as_ref())?;
                        let value = std::str::from_utf8(&attr.value)?;
                        self.attributes.insert(key.to_string(), value.to_string());
                    }

                    self.text_buffer.clear();

                    // State transitions using builders
                    match (&self.state, name) {
                        (ParserState::Initial, "MessageHeader") => {
                            self.state =
                                ParserState::InHeader(Box::new(MessageHeaderBuilder::new()));
                        }
                        (ParserState::Initial, "Release") => {
                            let reference = self
                                .attributes
                                .get("ReleaseReference")
                                .unwrap_or(&"default".to_string())
                                .clone();
                            let release = ReleaseBuilder::new(reference);
                            self.state = ParserState::InRelease(Box::new(release));
                        }
                        (ParserState::Initial, "Resource") => {
                            let reference = self
                                .attributes
                                .get("ResourceReference")
                                .unwrap_or(&"default".to_string())
                                .clone();
                            let resource = ResourceBuilder::new(reference);
                            self.state = ParserState::InResource(Box::new(resource));
                        }
                        (ParserState::Initial, "Party") => {
                            self.state = ParserState::InParty(Box::new(PartyBuilder::new(None)));
                        }
                        _ => {
                            // Continue in current state
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    let name_bytes = e.name();
                    let name = std::str::from_utf8(name_bytes.as_ref())?;
                    let text_content = self.text_buffer.clone();
                    let bytes_processed = self.bytes_processed;

                    let location = format!("streaming at byte offset {}", bytes_processed);

                    let result = match &mut self.state {
                        ParserState::InHeader(header) => {
                            match name {
                                "MessageId" => {
                                    header.set_message_id(text_content.clone());
                                    None
                                }
                                "MessageCreatedDateTime" => {
                                    header.set_created_date_time_from_text(text_content.clone());
                                    None
                                }
                                "MessageHeader" => {
                                    // Complete header - convert using ToCore trait
                                    let core_header = header.clone().to_core().map_err(|e| {
                                        ParseError::ConversionError {
                                            from: "StreamingHeader".to_string(),
                                            to: "MessageHeader".to_string(),
                                            message: format!(
                                                "Failed to convert header at {}: {:?}",
                                                location, e
                                            ),
                                        }
                                    })?;
                                    self.state = ParserState::Initial;
                                    Some(StreamingElement::Header(Box::new(core_header)))
                                }
                                _ => None,
                            }
                        }
                        ParserState::InRelease(release) => {
                            match name {
                                "ReleaseTitle" => {
                                    release.add_title(LocalizedString {
                                        text: text_content.clone(),
                                        language_code: None,
                                        script: None,
                                    });
                                    None
                                }
                                "Genre" => {
                                    let genre = Genre {
                                        genre_text: text_content.clone(),
                                        sub_genre: None,
                                        attributes: None,
                                        extensions: None,
                                        comments: None,
                                    };
                                    release.add_genre(genre);
                                    None
                                }
                                "Release" => {
                                    // Complete release - convert using ToCore trait
                                    let core_release = release.clone().to_core().map_err(|e| {
                                        ParseError::ConversionError {
                                            from: "StreamingRelease".to_string(),
                                            to: "Release".to_string(),
                                            message: format!(
                                                "Failed to convert release at {}: {:?}",
                                                location, e
                                            ),
                                        }
                                    })?;
                                    self.state = ParserState::Initial;
                                    Some(StreamingElement::Release(core_release))
                                }
                                _ => None,
                            }
                        }
                        ParserState::InResource(resource) => {
                            match name {
                                "Title" | "ReferenceTitle" => {
                                    resource.add_title(LocalizedString {
                                        text: text_content.clone(),
                                        language_code: None,
                                        script: None,
                                    });
                                    None
                                }
                                "Duration" => {
                                    resource.set_duration_from_text(text_content.clone());
                                    None
                                }
                                "Resource" => {
                                    // Complete resource - convert using ToCore trait
                                    let core_resource =
                                        resource.clone().to_core().map_err(|e| {
                                            ParseError::ConversionError {
                                                from: "StreamingResource".to_string(),
                                                to: "Resource".to_string(),
                                                message: format!(
                                                    "Failed to convert resource at {}: {:?}",
                                                    location, e
                                                ),
                                            }
                                        })?;
                                    self.state = ParserState::Initial;
                                    Some(StreamingElement::Resource(core_resource))
                                }
                                _ => None,
                            }
                        }
                        ParserState::InParty(party) => {
                            match name {
                                "PartyName" => {
                                    party.add_name(LocalizedString {
                                        text: text_content.clone(),
                                        language_code: None,
                                        script: None,
                                    });
                                    None
                                }
                                "Party" => {
                                    // Complete party - convert using ToCore trait
                                    let core_party = party.clone().to_core().map_err(|e| {
                                        ParseError::ConversionError {
                                            from: "StreamingParty".to_string(),
                                            to: "Party".to_string(),
                                            message: format!(
                                                "Failed to convert party at {}: {:?}",
                                                location, e
                                            ),
                                        }
                                    })?;
                                    self.state = ParserState::Initial;
                                    Some(StreamingElement::Party(core_party))
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    };

                    self.current_depth = self.current_depth.saturating_sub(1);
                    self.current_path.pop();
                    self.text_buffer.clear();

                    if let Some(element) = result {
                        self.elements_yielded += 1;
                        return Ok(Some(element));
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = std::str::from_utf8(&e)?;
                    self.text_buffer.push_str(text.trim());
                }
                Ok(Event::Eof) => {
                    return Ok(Some(StreamingElement::EndOfStream));
                }
                Ok(_) => {
                    // Skip other events
                }
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error: {}", e)));
                }
            }

            self.bytes_processed = bytes_position;

            // Check security limits
            if self.current_depth > 100 {
                return Err(ParseError::SecurityViolation {
                    message: "Nesting depth exceeds 100 levels".to_string(),
                });
            }
        }
    }

    // Helper methods for error handling

    fn get_current_location(&self) -> String {
        format!("streaming at byte offset {}", self.bytes_processed)
    }

    pub fn stats(&self) -> ComprehensiveStats {
        ComprehensiveStats {
            bytes_processed: self.bytes_processed,
            elements_yielded: self.elements_yielded,
            current_depth: self.current_depth,
            elapsed: self.start_time.elapsed(),
        }
    }
}

/// Iterator wrapper for comprehensive streaming parser
pub struct ComprehensiveStreamIterator<R: BufRead> {
    parser: ComprehensiveStreamingParser<R>,
    finished: bool,
}

impl<R: BufRead> ComprehensiveStreamIterator<R> {
    pub fn new(reader: R, version: ERNVersion) -> Self {
        Self {
            parser: ComprehensiveStreamingParser::new(reader, version),
            finished: false,
        }
    }

    pub fn stats(&self) -> ComprehensiveStats {
        self.parser.stats()
    }
}

impl<R: BufRead> Iterator for ComprehensiveStreamIterator<R> {
    type Item = Result<StreamingElement, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.parser.parse_next() {
            Ok(Some(element)) => {
                if matches!(element, StreamingElement::EndOfStream) {
                    self.finished = true;
                }
                Some(Ok(element))
            }
            Ok(None) => {
                self.finished = true;
                None
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComprehensiveStats {
    pub bytes_processed: u64,
    pub elements_yielded: usize,
    pub current_depth: usize,
    pub elapsed: std::time::Duration,
}

impl ComprehensiveStats {
    pub fn throughput_mibs(&self) -> f64 {
        if self.elapsed.as_secs_f64() > 0.0 {
            (self.bytes_processed as f64 / (1024.0 * 1024.0)) / self.elapsed.as_secs_f64()
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_comprehensive_streaming_parser() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ERNMessage xmlns="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>test-message-1</MessageId>
        <MessageCreatedDateTime>2023-01-01T00:00:00</MessageCreatedDateTime>
    </MessageHeader>
    <Release ReleaseReference="REL001">
        <ReleaseTitle>Test Release</ReleaseTitle>
        <Genre>Rock</Genre>
    </Release>
    <Resource ResourceReference="RES001">
        <Title>Test Resource</Title>
        <Duration>180</Duration>
    </Resource>
</ERNMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let iterator = ComprehensiveStreamIterator::new(cursor, ERNVersion::V4_3);

        let elements: Result<Vec<_>, _> = iterator.collect();
        if let Err(ref e) = elements {
            error!("Iterator failed to collect elements: {:?}", e);
        }
        assert!(
            elements.is_ok(),
            "Iterator failed with error: {:?}",
            elements.as_ref().err()
        );

        let elements = elements.unwrap();
        assert!(elements.len() >= 3); // Header, Release, Resource, EndOfStream

        // Check we got the expected elements
        let has_header = elements
            .iter()
            .any(|e| matches!(e, StreamingElement::Header(_)));
        let has_release = elements
            .iter()
            .any(|e| matches!(e, StreamingElement::Release(_)));
        let has_resource = elements
            .iter()
            .any(|e| matches!(e, StreamingElement::Resource(_)));
        let has_end_stream = elements
            .iter()
            .any(|e| matches!(e, StreamingElement::EndOfStream));

        assert!(has_header, "Should parse message header");
        assert!(has_release, "Should parse release");
        assert!(has_resource, "Should parse resource");
        assert!(has_end_stream, "Should have end of stream marker");
    }

    #[test]
    fn test_comprehensive_security_limits() {
        // Create deeply nested XML
        let mut xml = String::from(r#"<?xml version="1.0"?>"#);
        for i in 0..150 {
            xml.push_str(&format!("<level{}>", i));
        }
        xml.push_str("content");
        for i in (0..150).rev() {
            xml.push_str(&format!("</level{}>", i));
        }

        let cursor = Cursor::new(xml.as_bytes());
        let mut iterator = ComprehensiveStreamIterator::new(cursor, ERNVersion::V4_3);

        // Should get a security violation
        let result = iterator.next();
        assert!(result.is_some());
        match result.unwrap() {
            Err(ParseError::SecurityViolation { .. }) => {
                // Expected
            }
            _ => panic!("Expected security violation"),
        }
    }
}
