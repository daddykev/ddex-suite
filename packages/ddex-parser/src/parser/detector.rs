use crate::error::ParseError;
use ddex_core::models::versions::ERNVersion;
use quick_xml::{events::Event, Reader};
use std::io::BufRead;
// core/src/parser/detector.rs

pub struct VersionDetector;

impl VersionDetector {
    pub fn detect<R: std::io::Read>(reader: R) -> crate::error::Result<ERNVersion> {
        let mut buf_reader = std::io::BufReader::new(reader);
        Self::detect_from_bufread(&mut buf_reader)
    }

    pub fn detect_from_bufread<R: BufRead>(reader: R) -> crate::error::Result<ERNVersion> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut found_root = false;
        let mut namespace_uris = Vec::new();

        // Parse XML and collect namespace URIs from the root element
        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    found_root = true;

                    // Extract namespace URIs from attributes
                    for attr in e.attributes() {
                        match attr {
                            Ok(attr) => {
                                let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                                let value = std::str::from_utf8(&attr.value).unwrap_or("");

                                // Look for xmlns declarations
                                if key == "xmlns" || key.starts_with("xmlns:") {
                                    namespace_uris.push(value.to_string());
                                }
                            }
                            Err(e) => {
                                return Err(ParseError::XmlError(format!("Invalid XML attribute: {}", e)));
                            }
                        }
                    }
                    break; // Only need the root element
                }
                Ok(Event::Eof) => {
                    break;
                }
                Ok(_) => {} // Skip other events
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error: {}", e)));
                }
            }
            buf.clear();
        }

        // If no root element found, it's invalid XML
        if !found_root {
            return Err(ParseError::XmlError("No root element found - invalid XML".to_string()));
        }

        // Check for DDEX ERN version in namespace URIs
        for uri in &namespace_uris {
            // ERN 3.8.1 is treated as 3.8.2 (minor differences, compatible structure)
            if uri.contains("http://ddex.net/xml/ern/381") || uri.contains("http://ddex.net/xml/ern/382") {
                return Ok(ERNVersion::V3_8_2);
            } else if uri.contains("http://ddex.net/xml/ern/42") {
                return Ok(ERNVersion::V4_2);
            } else if uri.contains("http://ddex.net/xml/ern/43") {
                return Ok(ERNVersion::V4_3);
            }
        }

        // If no DDEX ERN namespace found, it's not a valid DDEX document
        Err(ParseError::XmlError("No DDEX ERN namespace found - not a valid DDEX document".to_string()))
    }
}
