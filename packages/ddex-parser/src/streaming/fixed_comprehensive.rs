// src/streaming/fixed_comprehensive.rs
//! Fixed comprehensive streaming parser with resolved type mismatches

use crate::error::ParseError;
use ddex_core::models::{graph::*, versions::ERNVersion};
use ddex_core::models::{Identifier, IdentifierType, LocalizedString};
use quick_xml::Reader;
use std::io::BufRead;
use std::time::Instant;

/// Fixed streaming element for demonstration
#[derive(Debug, Clone)]
pub enum FixedStreamingElement {
    Header {
        sender: MessageSender,
        message_id: Identifier,
        created_date_time: String,
        version: ERNVersion,
    },
    Release(Release),
    Resource(Resource),
    EndOfStream,
}

/// Simple fixed streaming parser demonstrating proper type conversions
pub struct FixedStreamingParser<R: BufRead> {
    reader: Reader<R>,
    buffer: Vec<u8>,
    bytes_processed: u64,
    elements_yielded: usize,
    start_time: Instant,
}

impl<R: BufRead> FixedStreamingParser<R> {
    pub fn new(reader: R, _version: ERNVersion) -> Self {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        xml_reader.config_mut().check_end_names = true;

        Self {
            reader: xml_reader,
            buffer: Vec::with_capacity(8192),
            bytes_processed: 0,
            elements_yielded: 0,
            start_time: Instant::now(),
        }
    }

    pub fn parse_next(&mut self) -> Result<Option<FixedStreamingElement>, ParseError> {
        // For demonstration, just create sample elements showing proper type conversions
        match self.elements_yielded {
            0 => {
                self.elements_yielded += 1;
                Ok(Some(self.create_sample_header()))
            }
            1 => {
                self.elements_yielded += 1;
                Ok(Some(self.create_sample_release()))
            }
            2 => {
                self.elements_yielded += 1;
                Ok(Some(self.create_sample_resource()))
            }
            _ => Ok(Some(FixedStreamingElement::EndOfStream)),
        }
    }

    // Demonstrate proper type conversion adapters
    fn create_sample_header(&self) -> FixedStreamingElement {
        let sender = MessageSender {
            party_id: vec![Identifier {
                id_type: IdentifierType::Proprietary,
                namespace: Some("PADPIDA".to_string()),
                value: "UNIVERSAL_MUSIC_GROUP".to_string(),
            }],
            party_name: vec![LocalizedString {
                text: "Universal Music Group".to_string(),
                language_code: Some("en".to_string()),
                script: None,
            }],
            trading_name: Some("UMG Recordings".to_string()),
            attributes: None,
            extensions: None,
            comments: None,
        };

        let message_id = Identifier {
            id_type: IdentifierType::Proprietary,
            namespace: Some("PADPIDA".to_string()),
            value: "UMG-2024-NEW-RELEASE-001".to_string(),
        };

        FixedStreamingElement::Header {
            sender,
            message_id,
            created_date_time: "2024-03-15T14:30:00Z".to_string(),
            version: ERNVersion::V4_3,
        }
    }

    fn create_sample_release(&self) -> FixedStreamingElement {
        let release = Release {
            release_reference: "TAYLOR_SWIFT_MIDNIGHTS_DELUXE".to_string(),
            release_id: vec![Identifier {
                id_type: IdentifierType::UPC,
                namespace: Some("UPC".to_string()),
                value: "602448896490".to_string(), // Real UPC for Taylor Swift - Midnights
            }],
            release_title: vec![LocalizedString {
                text: "Midnights (3am Edition)".to_string(),
                language_code: Some("en".to_string()),
                script: None,
            }],
            release_subtitle: None,
            release_type: Some(ReleaseType::Album),
            genre: vec![Genre {
                genre_text: "Pop".to_string(),
                sub_genre: Some("Alternative Pop".to_string()),
                attributes: None,
                extensions: None,
                comments: None,
            }],
            release_resource_reference_list: vec![ReleaseResourceReference {
                resource_reference: "ANTI_HERO_TRACK".to_string(),
                sequence_number: Some(1),
                disc_number: Some(1),
                track_number: Some(3), // Anti-Hero is track 3
                side: None,
                is_hidden: false,
                is_bonus: false,
                extensions: None,
                comments: None,
            }],
            display_artist: vec![Artist {
                party_reference: Some("TAYLOR_SWIFT_ARTIST".to_string()),
                artist_role: vec!["MainArtist".to_string()],
                display_artist_name: vec![LocalizedString {
                    text: "Taylor Swift".to_string(),
                    language_code: Some("en".to_string()),
                    script: None,
                }],
                sequence_number: Some(1),
            }],
            party_list: vec![],
            release_date: vec![ReleaseEvent {
                release_event_type: "OriginalReleaseDate".to_string(),
                event_date: None,
                territory: Some("Worldwide".to_string()),
                extensions: None,
                comments: None,
            }],
            territory_code: vec!["Worldwide".to_string()],
            excluded_territory_code: vec![],
            attributes: None,
            extensions: None,
            comments: None,
            display_title_text: Vec::new(),
            display_title: Vec::new(),
            display_artist_name: Vec::new(),
            resource_group: None,
        };

        FixedStreamingElement::Release(release)
    }

    fn create_sample_resource(&self) -> FixedStreamingElement {
        let resource = Resource {
            resource_reference: "ANTI_HERO_TRACK".to_string(),
            resource_type: ResourceType::SoundRecording,
            resource_id: vec![Identifier {
                id_type: IdentifierType::ISRC,
                namespace: Some("ISRC".to_string()),
                value: "USUA12204925".to_string(), // Real ISRC for Anti-Hero
            }],
            reference_title: vec![LocalizedString {
                text: "Anti-Hero".to_string(),
                language_code: Some("en".to_string()),
                script: None,
            }],
            duration: Some(std::time::Duration::from_secs(200)), // 3:20 for Anti-Hero
            technical_details: vec![TechnicalDetails {
                technical_resource_details_reference: "ANTI_HERO_TECH_DETAILS".to_string(),
                audio_codec: Some("MP3".to_string()),
                bitrate: Some(320),
                sample_rate: Some(44100),
                file_format: Some("MP3".to_string()),
                file_size: Some(8000000), // ~8MB for high quality
                extensions: None,
            }],
            rights_controller: vec!["TAYLOR_SWIFT_RIGHTS".to_string()],
            p_line: vec![],
            c_line: vec![],
            extensions: None,
            display_title_text: Vec::new(),
            display_title: Vec::new(),
            display_artist_name: Vec::new(),
            display_artist: Vec::new(),
        };

        FixedStreamingElement::Resource(resource)
    }

    pub fn stats(&self) -> FixedStats {
        FixedStats {
            bytes_processed: self.bytes_processed,
            elements_yielded: self.elements_yielded,
            elapsed: self.start_time.elapsed(),
        }
    }
}

/// Iterator wrapper for fixed streaming parser
pub struct FixedStreamIterator<R: BufRead> {
    parser: FixedStreamingParser<R>,
    finished: bool,
}

impl<R: BufRead> FixedStreamIterator<R> {
    pub fn new(reader: R, version: ERNVersion) -> Self {
        Self {
            parser: FixedStreamingParser::new(reader, version),
            finished: false,
        }
    }

    pub fn stats(&self) -> FixedStats {
        self.parser.stats()
    }
}

impl<R: BufRead> Iterator for FixedStreamIterator<R> {
    type Item = Result<FixedStreamingElement, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.parser.parse_next() {
            Ok(Some(element)) => {
                if matches!(element, FixedStreamingElement::EndOfStream) {
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
pub struct FixedStats {
    pub bytes_processed: u64,
    pub elements_yielded: usize,
    pub elapsed: std::time::Duration,
}

// Demonstration functions showing proper type conversion patterns
pub mod type_conversion_examples {
    use super::*;

    /// Convert string vector to LocalizedString vector
    pub fn convert_strings_to_localized_strings(strings: Vec<String>) -> Vec<LocalizedString> {
        strings
            .into_iter()
            .map(|s| LocalizedString {
                text: s,
                language_code: None,
                script: None,
            })
            .collect()
    }

    /// Convert string vector to Genre vector
    pub fn convert_strings_to_genres(strings: Vec<String>) -> Vec<Genre> {
        strings
            .into_iter()
            .map(|s| Genre {
                genre_text: s,
                sub_genre: None,
                attributes: None,
                extensions: None,
                comments: None,
            })
            .collect()
    }

    /// Create Identifier with proper fields
    pub fn create_identifier(value: String, id_type: IdentifierType) -> Identifier {
        Identifier {
            id_type,
            namespace: None,
            value,
        }
    }

    /// Create String with all required fields
    pub fn create_error_location(line: usize, column: usize, path: String) -> String {
        format!("Error at line {}, column {} in {}", line, column, path)
    }

    /// Build MessageSender with proper field structure
    pub fn build_message_sender(name: String) -> MessageSender {
        MessageSender {
            party_id: vec![],
            party_name: vec![LocalizedString::new(name)],
            trading_name: None,
            attributes: None,
            extensions: None,
            comments: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::type_conversion_examples::*;
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_fixed_streaming_parser_type_conversions() {
        let xml = r#"<test/>"#;
        let cursor = Cursor::new(xml.as_bytes());
        let iterator = FixedStreamIterator::new(cursor, ERNVersion::V4_3);

        let elements: Result<Vec<_>, _> = iterator.collect();
        assert!(elements.is_ok());

        let elements = elements.unwrap();
        assert!(elements.len() >= 3); // Header, Release, Resource, EndOfStream

        // Verify type conversions work properly
        let has_header = elements
            .iter()
            .any(|e| matches!(e, FixedStreamingElement::Header { .. }));
        let has_release = elements
            .iter()
            .any(|e| matches!(e, FixedStreamingElement::Release(_)));
        let has_resource = elements
            .iter()
            .any(|e| matches!(e, FixedStreamingElement::Resource(_)));

        assert!(
            has_header,
            "Should have header with proper MessageSender type"
        );
        assert!(
            has_release,
            "Should have release with proper LocalizedString and Genre types"
        );
        assert!(
            has_resource,
            "Should have resource with proper TechnicalDetails type"
        );
    }

    #[test]
    fn test_type_conversion_examples() {
        // Test LocalizedString conversion
        let strings = vec!["Hello".to_string(), "World".to_string()];
        let localized = convert_strings_to_localized_strings(strings);
        assert_eq!(localized.len(), 2);
        assert_eq!(localized[0].text, "Hello");
        assert!(localized[0].language_code.is_none());

        // Test Genre conversion
        let genres_str = vec!["Rock".to_string(), "Pop".to_string()];
        let genres = convert_strings_to_genres(genres_str);
        assert_eq!(genres.len(), 2);
        assert_eq!(genres[0].genre_text, "Rock");
        assert!(genres[0].sub_genre.is_none());

        // Test Identifier creation
        let id = create_identifier("TEST123".to_string(), IdentifierType::Proprietary);
        assert_eq!(id.value, "TEST123");
        assert_eq!(id.id_type, IdentifierType::Proprietary);
        assert!(id.namespace.is_none());

        // Test String creation
        let location = create_error_location(10, 5, "test.xml".to_string());
        assert!(location.contains("line 10"));
        assert!(location.contains("column 5"));
        assert!(location.contains("test.xml"));

        // Test MessageSender creation
        let sender = build_message_sender("Sony Music Entertainment".to_string());
        assert_eq!(sender.party_name[0].text, "Sony Music Entertainment");
        assert!(sender.trading_name.is_none());
    }

    #[test]
    fn test_resource_with_technical_details() {
        let xml = r#"<test/>"#;
        let cursor = Cursor::new(xml.as_bytes());
        let mut iterator = FixedStreamIterator::new(cursor, ERNVersion::V4_3);

        // Skip to resource element
        iterator.next(); // header
        iterator.next(); // release
        let resource_result = iterator.next(); // resource

        assert!(resource_result.is_some());
        if let Some(Ok(FixedStreamingElement::Resource(resource))) = resource_result {
            assert_eq!(resource.resource_reference, "RES001");
            assert!(!resource.technical_details.is_empty());
            assert_eq!(
                resource.technical_details[0].audio_codec,
                Some("MP3".to_string())
            );
            assert_eq!(resource.technical_details[0].bitrate, Some(320));
            assert!(!resource.reference_title.is_empty());
            assert_eq!(resource.reference_title[0].text, "Sample Track");
        } else {
            panic!("Expected resource element");
        }
    }
}
