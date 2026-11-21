// src/streaming/aligned_comprehensive.rs
//! Model-aligned comprehensive streaming parser using builders

#[allow(dead_code)] // Experimental streaming parser implementation
use crate::error::ParseError;
use ddex_core::models::streaming_types::builders::*;
use ddex_core::models::streaming_types::*;
use ddex_core::models::IdentifierType;
use ddex_core::models::{graph::*, versions::ERNVersion};
use log::warn;
use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};
use std::collections::HashMap;
use std::io::BufRead;
use std::time::Instant;

/// Aligned streaming element using proper core types
#[derive(Debug, Clone)]
pub enum AlignedStreamingElement {
    Header(Box<MessageHeader>),
    Release(Release),
    Resource(Resource),
    Party(Party),
    EndOfStream,
}

/// Parser state using builder pattern
#[derive(Debug)]
enum AlignedParserState {
    Initial,
    InHeader(Box<MessageHeaderBuilder>),
    InRelease(Box<ReleaseBuilder>),
    InResource(Box<ResourceBuilder>),
    InParty(Box<PartyBuilder>),
    Complete,
}

/// Model-aligned streaming parser
pub struct AlignedStreamingParser<R: BufRead> {
    reader: Reader<R>,
    buffer: Vec<u8>,
    state: AlignedParserState,
    current_path: Vec<String>,
    current_depth: usize,
    text_buffer: String,
    attributes: HashMap<String, String>,
    bytes_processed: u64,
    elements_yielded: usize,
    start_time: Instant,
}

impl<R: BufRead> AlignedStreamingParser<R> {
    pub fn new(reader: R, _version: ERNVersion) -> Self {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        xml_reader.config_mut().check_end_names = true;

        Self {
            reader: xml_reader,
            buffer: Vec::with_capacity(8192),
            state: AlignedParserState::Initial,
            current_path: Vec::new(),
            current_depth: 0,
            text_buffer: String::new(),
            attributes: HashMap::new(),
            bytes_processed: 0,
            elements_yielded: 0,
            start_time: Instant::now(),
        }
    }

    pub fn parse_next(&mut self) -> Result<Option<AlignedStreamingElement>, ParseError> {
        loop {
            self.buffer.clear();
            let event = self.reader.read_event_into(&mut self.buffer)?;
            match event {
                Event::Start(e) => {
                    let name_bytes = e.name();
                    let name = std::str::from_utf8(name_bytes.as_ref())?.to_string();

                    // Extract attributes into a temporary structure
                    let mut attributes = HashMap::new();
                    for attr_result in e.attributes() {
                        let attr = attr_result
                            .map_err(|e| ParseError::XmlError(format!("Attribute error: {}", e)))?;

                        let key = std::str::from_utf8(attr.key.as_ref())?;
                        let value = std::str::from_utf8(&attr.value)?;

                        attributes.insert(key.to_string(), value.to_string());
                    }

                    // Store the attributes
                    self.attributes = attributes;

                    self.handle_start_element_by_name(&name)?;
                }
                Event::End(e) => {
                    let name_bytes = e.name();
                    let name = std::str::from_utf8(name_bytes.as_ref())?.to_string();
                    if let Some(element) = self.handle_end_element_by_name(&name)? {
                        self.elements_yielded += 1;
                        return Ok(Some(element));
                    }
                }
                Event::Text(e) => {
                    let text = std::str::from_utf8(&e)?;
                    self.text_buffer.push_str(text.trim());
                }
                Event::Eof => {
                    return Ok(Some(AlignedStreamingElement::EndOfStream));
                }
                _ => {
                    // Skip other events
                }
            }

            self.bytes_processed = self.reader.buffer_position();

            // Check security limits
            if self.current_depth > 100 {
                return Err(ParseError::SecurityViolation {
                    message: "Nesting depth exceeds 100 levels".to_string(),
                });
            }

            self.buffer.clear();
        }
    }

    fn handle_start_element_by_name(&mut self, name: &str) -> Result<(), ParseError> {
        self.current_path.push(name.to_string());
        self.current_depth += 1;

        self.text_buffer.clear();

        // State transitions using builders
        match (&self.state, name) {
            (AlignedParserState::Initial, "MessageHeader") => {
                self.state = AlignedParserState::InHeader(Box::new(MessageHeaderBuilder::new()));
            }
            (AlignedParserState::Initial, "Release") => {
                let reference = self
                    .attributes
                    .get("ReleaseReference")
                    .cloned()
                    .unwrap_or_else(|| format!("REL_{}", self.elements_yielded));
                self.state =
                    AlignedParserState::InRelease(Box::new(ReleaseBuilder::new(reference)));
            }
            (AlignedParserState::Initial, "Resource") => {
                let reference = self
                    .attributes
                    .get("ResourceReference")
                    .cloned()
                    .unwrap_or_else(|| format!("RES_{}", self.elements_yielded));
                self.state =
                    AlignedParserState::InResource(Box::new(ResourceBuilder::new(reference)));
            }
            (AlignedParserState::Initial, "Party") => {
                let reference = self.attributes.get("PartyReference").cloned();
                self.state = AlignedParserState::InParty(Box::new(PartyBuilder::new(reference)));
            }
            _ => {
                // Continue in current state
            }
        }

        Ok(())
    }

    fn handle_end_element_by_name(
        &mut self,
        name: &str,
    ) -> Result<Option<AlignedStreamingElement>, ParseError> {
        let text_content = self.text_buffer.clone();

        let result = match &mut self.state {
            AlignedParserState::InHeader(builder) => {
                match name {
                    "MessageId" => {
                        builder.set_message_id(text_content);
                        None
                    }
                    "MessageCreatedDateTime" => {
                        builder.set_created_date_time_from_text(text_content);
                        None
                    }
                    "MessageSender" => {
                        // For simplicity, create a basic sender
                        let sender = create_message_sender(
                            text_content.clone(),
                            Some(format!("SENDER_{}", self.elements_yielded)),
                        );
                        builder.set_sender(sender);
                        None
                    }
                    "MessageRecipient" => {
                        let recipient = create_message_recipient(text_content);
                        builder.set_recipient(recipient);
                        None
                    }
                    "MessageHeader" => {
                        // Complete header - use builder to create element
                        let builder =
                            std::mem::replace(&mut self.state, AlignedParserState::Initial);
                        if let AlignedParserState::InHeader(header_builder) = builder {
                            match header_builder.to_core() {
                                Ok(header) => {
                                    Some(AlignedStreamingElement::Header(Box::new(header)))
                                }
                                Err(e) => {
                                    warn!("Header validation failed, using fallback: {}", e);
                                    // Create a minimal valid header
                                    let header = self.create_fallback_header();
                                    Some(AlignedStreamingElement::Header(Box::new(header)))
                                }
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            AlignedParserState::InRelease(builder) => {
                match name {
                    "ReleaseTitle" => {
                        let title = create_localized_string(
                            text_content,
                            self.attributes.get("LanguageCode").cloned(),
                        );
                        builder.add_title(title);
                        None
                    }
                    "Genre" => {
                        let genre = create_genre(text_content, None);
                        builder.add_genre(genre);
                        None
                    }
                    "DisplayArtist" => {
                        let artist = create_artist(text_content, "MainArtist".to_string(), None);
                        builder.add_artist(artist);
                        None
                    }
                    "ReleaseType" => {
                        let release_type = match text_content.as_str() {
                            "Album" => ReleaseType::Album,
                            "Single" => ReleaseType::Single,
                            "EP" => ReleaseType::EP,
                            _ => ReleaseType::Other(text_content),
                        };
                        builder.set_release_type(release_type);
                        None
                    }
                    "Release" => {
                        // Complete release - use builder
                        let builder =
                            std::mem::replace(&mut self.state, AlignedParserState::Initial);
                        if let AlignedParserState::InRelease(release_builder) = builder {
                            match release_builder.to_core() {
                                Ok(release) => Some(AlignedStreamingElement::Release(release)),
                                Err(e) => {
                                    warn!("Release validation failed, skipping release: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            AlignedParserState::InResource(builder) => {
                match name {
                    "Title" => {
                        let title = create_localized_string(
                            text_content,
                            self.attributes.get("LanguageCode").cloned(),
                        );
                        builder.add_title(title);
                        None
                    }
                    "Duration" => {
                        builder.set_duration_from_text(text_content);
                        None
                    }
                    "ResourceType" => {
                        let resource_type = match text_content.as_str() {
                            "SoundRecording" => ResourceType::SoundRecording,
                            "Video" => ResourceType::Video,
                            "Image" => ResourceType::Image,
                            "Text" => ResourceType::Text,
                            "SheetMusic" => ResourceType::SheetMusic,
                            _ => ResourceType::SoundRecording, // Default
                        };
                        builder.set_resource_type(resource_type);
                        None
                    }
                    "ISRC" => {
                        let identifier = create_identifier(
                            text_content,
                            IdentifierType::ISRC,
                            Some("ISRC".to_string()),
                        );
                        builder.add_identifier(identifier);
                        None
                    }
                    "Resource" => {
                        // Complete resource - use builder
                        let builder =
                            std::mem::replace(&mut self.state, AlignedParserState::Initial);
                        if let AlignedParserState::InResource(resource_builder) = builder {
                            match resource_builder.to_core() {
                                Ok(resource) => Some(AlignedStreamingElement::Resource(resource)),
                                Err(e) => {
                                    warn!("Resource validation failed, skipping resource: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            AlignedParserState::InParty(builder) => {
                match name {
                    "PartyName" => {
                        let name = create_localized_string(
                            text_content,
                            self.attributes.get("LanguageCode").cloned(),
                        );
                        builder.add_name(name);
                        None
                    }
                    "ISNI" => {
                        builder.set_isni(text_content);
                        None
                    }
                    "PartyRole" => {
                        let role = match text_content.as_str() {
                            "Artist" => PartyRole::Artist,
                            "Producer" => PartyRole::Producer,
                            "Composer" => PartyRole::Composer,
                            "Lyricist" => PartyRole::Lyricist,
                            "Publisher" => PartyRole::Publisher,
                            "Performer" => PartyRole::Performer,
                            "Engineer" => PartyRole::Engineer,
                            "Label" => PartyRole::Label,
                            "Distributor" => PartyRole::Distributor,
                            _ => PartyRole::Other(text_content),
                        };
                        builder.add_role(role);
                        None
                    }
                    "Party" => {
                        // Complete party - use builder
                        let builder =
                            std::mem::replace(&mut self.state, AlignedParserState::Initial);
                        if let AlignedParserState::InParty(party_builder) = builder {
                            match party_builder.to_core() {
                                Ok(party) => Some(AlignedStreamingElement::Party(party)),
                                Err(e) => {
                                    warn!("Party validation failed, skipping party: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        };

        self.current_depth = self.current_depth.saturating_sub(1);
        self.current_path.pop();
        self.text_buffer.clear();

        Ok(result)
    }

    fn create_fallback_header(&self) -> MessageHeader {
        MessageHeader {
            message_id: "FALLBACK_MSG".to_string(),
            message_type: MessageType::NewReleaseMessage,
            message_created_date_time: chrono::Utc::now(),
            message_sender: create_message_sender("Unknown Sender".to_string(), None),
            message_recipient: create_message_recipient("Unknown Recipient".to_string()),
            message_control_type: None,
            message_thread_id: None,
            attributes: None,
            extensions: None,
            comments: None,
        }
    }

    fn get_current_location(&self) -> String {
        format!("aligned_streaming:{}:0", self.bytes_processed)
    }

    pub fn stats(&self) -> AlignedStats {
        AlignedStats {
            bytes_processed: self.bytes_processed,
            elements_yielded: self.elements_yielded,
            current_depth: self.current_depth,
            elapsed: self.start_time.elapsed(),
        }
    }
}

/// Iterator wrapper for aligned streaming parser
pub struct AlignedStreamIterator<R: BufRead> {
    parser: AlignedStreamingParser<R>,
    finished: bool,
}

impl<R: BufRead> AlignedStreamIterator<R> {
    pub fn new(reader: R, version: ERNVersion) -> Self {
        Self {
            parser: AlignedStreamingParser::new(reader, version),
            finished: false,
        }
    }

    pub fn stats(&self) -> AlignedStats {
        self.parser.stats()
    }
}

impl<R: BufRead> Iterator for AlignedStreamIterator<R> {
    type Item = Result<AlignedStreamingElement, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.parser.parse_next() {
            Ok(Some(element)) => {
                if matches!(element, AlignedStreamingElement::EndOfStream) {
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
pub struct AlignedStats {
    pub bytes_processed: u64,
    pub elements_yielded: usize,
    pub current_depth: usize,
    pub elapsed: std::time::Duration,
}

impl AlignedStats {
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
    fn test_aligned_streaming_parser_with_builders() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ERNMessage xmlns="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>UMG-2024-NEW-RELEASE-001</MessageId>
        <MessageCreatedDateTime>2024-03-15T14:30:00Z</MessageCreatedDateTime>
        <MessageSender>Universal Music Group</MessageSender>
        <MessageRecipient>Spotify Technology</MessageRecipient>
    </MessageHeader>
    <Release ReleaseReference="TAYLOR_SWIFT_MIDNIGHTS_DELUXE">
        <ReleaseTitle>Midnights (3am Edition)</ReleaseTitle>
        <Genre>Pop</Genre>
        <ReleaseType>Album</ReleaseType>
        <DisplayArtist>Taylor Swift</DisplayArtist>
    </Release>
    <Resource ResourceReference="ANTI_HERO_TRACK">
        <Title>Anti-Hero</Title>
        <Duration>200</Duration>
        <ResourceType>SoundRecording</ResourceType>
        <ISRC>USUA12204925</ISRC>
    </Resource>
    <Party PartyReference="TAYLOR_SWIFT_ARTIST">
        <PartyName>Taylor Swift</PartyName>
        <PartyRole>Artist</PartyRole>
        <ISNI>0000000368570204</ISNI>
    </Party>
</ERNMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let iterator = AlignedStreamIterator::new(cursor, ERNVersion::V4_3);

        let elements: Result<Vec<_>, _> = iterator.collect();
        assert!(elements.is_ok());

        let elements = elements.unwrap();
        assert!(elements.len() >= 4); // Header, Release, Resource, Party, EndOfStream

        // Verify proper type construction
        let has_header = elements
            .iter()
            .any(|e| matches!(e, AlignedStreamingElement::Header(_)));
        let has_release = elements
            .iter()
            .any(|e| matches!(e, AlignedStreamingElement::Release(_)));
        let has_resource = elements
            .iter()
            .any(|e| matches!(e, AlignedStreamingElement::Resource(_)));
        let has_party = elements
            .iter()
            .any(|e| matches!(e, AlignedStreamingElement::Party(_)));

        assert!(
            has_header,
            "Should parse message header using MessageHeaderBuilder"
        );
        assert!(has_release, "Should parse release using ReleaseBuilder");
        assert!(has_resource, "Should parse resource using ResourceBuilder");
        assert!(has_party, "Should parse party using PartyBuilder");

        // Verify field mapping
        for element in &elements {
            match element {
                AlignedStreamingElement::Header(header) => {
                    assert_eq!(header.message_id, "UMG-2024-NEW-RELEASE-001");
                    assert_eq!(
                        header.message_sender.party_name[0].text,
                        "Universal Music Group"
                    );
                }
                AlignedStreamingElement::Release(release) => {
                    assert_eq!(release.release_reference, "TAYLOR_SWIFT_MIDNIGHTS_DELUXE");
                    assert_eq!(release.release_title[0].text, "Midnights (3am Edition)");
                    assert_eq!(release.genre[0].genre_text, "Pop");
                    assert_eq!(release.release_type, Some(ReleaseType::Album));
                }
                AlignedStreamingElement::Resource(resource) => {
                    assert_eq!(resource.resource_reference, "ANTI_HERO_TRACK");
                    assert_eq!(resource.reference_title[0].text, "Anti-Hero");
                    assert_eq!(resource.duration, Some(std::time::Duration::from_secs(200)));
                    assert_eq!(resource.resource_type, ResourceType::SoundRecording);
                }
                AlignedStreamingElement::Party(party) => {
                    assert_eq!(party.party_name[0].text, "Taylor Swift");
                    assert_eq!(party.isni, Some("0000000368570204".to_string())); // Real ISNI for Taylor Swift
                    assert!(party.party_role.contains(&PartyRole::Artist));
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_builder_validation() {
        let xml = r#"<?xml version="1.0"?>
<ERNMessage>
    <Release>
        <!-- Missing required fields -->
    </Release>
</ERNMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let mut iterator = AlignedStreamIterator::new(cursor, ERNVersion::V4_3);

        // Should handle validation gracefully
        let elements: Vec<_> = iterator.collect();
        // Should not crash despite missing required fields
        assert!(!elements.is_empty());
    }

    #[test]
    fn test_conversion_traits() {
        // Test ToCore trait
        let mut builder = ReleaseBuilder::new("FOLKLORE_DELUXE".to_string());
        builder.add_title(create_localized_string(
            "Folklore (Deluxe Version)".to_string(),
            None,
        ));

        let release = builder.to_core().unwrap();
        assert_eq!(release.release_reference, "FOLKLORE_DELUXE");
        assert_eq!(release.release_title[0].text, "Folklore (Deluxe Version)");

        // Test Validate trait
        let empty_builder = ReleaseBuilder::default();
        assert!(!empty_builder.is_complete());
        assert!(empty_builder.validate().is_err());
    }
}
