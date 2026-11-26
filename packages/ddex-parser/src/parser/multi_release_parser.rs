// src/parser/multi_release_parser.rs
//! Enhanced multi-release parsing with accurate counting and content extraction

use crate::error::ParseError;
use crate::parser::security::SecurityConfig;
use ddex_core::models::graph::{MessageHeader, Release, ReleaseType};
use ddex_core::models::versions::ERNVersion;
use ddex_core::models::{Identifier, IdentifierType, LocalizedString};
use quick_xml::{events::Event, Reader};
use std::collections::HashMap;
use std::io::BufRead;

/// Enhanced multi-release parser with accurate counting and content extraction
#[derive(Debug, Clone)]
pub struct MultiReleaseParser {
    /// Security configuration
    security_config: SecurityConfig,
    /// Maximum releases to parse (0 = unlimited)
    max_releases: usize,
    /// Track parsing statistics
    stats: MultiReleaseStats,
}

/// Parsing statistics for multi-release documents
#[derive(Debug, Clone, Default)]
pub struct MultiReleaseStats {
    pub total_releases_found: usize,
    pub releases_parsed: usize,
    pub main_releases: usize,
    pub secondary_releases: usize,
    pub elements_processed: usize,
    pub bytes_processed: usize,
    pub parse_duration: std::time::Duration,
    pub release_list_count: usize,
}

/// Result of multi-release parsing
#[derive(Debug, Clone)]
pub struct MultiReleaseResult {
    /// Parsed releases (empty if count_only mode)
    pub releases: Vec<Release>,
    /// Parsing statistics
    pub stats: MultiReleaseStats,
    /// Not read in 0.5.x: always `None` (before 0.5.1, an invented header with the message
    /// ID `MSG_<version>` and the current time). `DDEXParser::parse` reads the header.
    pub message_header: Option<MessageHeader>,
    /// Raw release count (fast counting)
    pub release_count: usize,
    /// Release references found
    pub release_references: Vec<String>,
}

/// Release parsing context
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ReleaseContext {
    release: Release,
    depth: usize,
    current_element_path: Vec<String>,
    attributes: HashMap<String, String>,
    is_main_release: Option<bool>,
    position: usize,
}

impl MultiReleaseParser {
    /// Create a new multi-release parser (the version is not used in 0.5.x)
    pub fn new(_version: ERNVersion) -> Self {
        Self {
            security_config: SecurityConfig::default(),
            max_releases: 0,
            stats: MultiReleaseStats::default(),
        }
    }

    /// Create parser with custom security configuration
    pub fn with_security_config(_version: ERNVersion, security_config: SecurityConfig) -> Self {
        Self {
            security_config,
            max_releases: 0,
            stats: MultiReleaseStats::default(),
        }
    }

    /// No effect in 0.5.x (it only controlled the invented header removed in 0.5.1); kept
    /// for compatibility.
    pub fn detailed_parsing(self, _enabled: bool) -> Self {
        self
    }

    /// Set maximum number of releases to parse
    pub fn max_releases(mut self, max: usize) -> Self {
        self.max_releases = max;
        self
    }

    /// Fast count of releases in the document without full parsing
    pub fn count_releases<R: BufRead>(&mut self, reader: R) -> Result<usize, ParseError> {
        let start_time = std::time::Instant::now();
        let mut xml_reader = Reader::from_reader(crate::xml_text::Checked::new(reader));
        xml_reader.config_mut().trim_text(false); // Faster for counting

        let mut buf = Vec::new();
        let mut release_count = 0;
        let mut depth = 0;
        let mut elements_processed = 0;

        loop {
            let read = xml_reader.read_event_into(&mut buf);
            crate::xml_text::check(xml_reader.get_ref())?;
            match read {
                Ok(Event::Start(ref e)) => {
                    elements_processed += 1;
                    depth += 1;

                    // Security check
                    if depth > self.security_config.max_element_depth {
                        return Err(ParseError::DepthLimitExceeded {
                            depth,
                            limit: self.security_config.max_element_depth,
                        });
                    }

                    let element_name = self.extract_element_name(e.name().as_ref())?;
                    if element_name == "Release" || element_name.ends_with(":Release") {
                        release_count += 1;

                        // Early exit if we have a maximum
                        if self.max_releases > 0 && release_count >= self.max_releases {
                            break;
                        }
                    }
                }
                Ok(Event::End(_)) => {
                    depth = depth.saturating_sub(1);
                }
                Ok(Event::Empty(ref e)) => {
                    elements_processed += 1;
                    let element_name = self.extract_element_name(e.name().as_ref())?;
                    if element_name == "Release" || element_name.ends_with(":Release") {
                        release_count += 1;

                        if self.max_releases > 0 && release_count >= self.max_releases {
                            break;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error: {}", e)));
                }
                _ => {} // Skip other events for speed
            }
            buf.clear();
        }

        // Update statistics
        self.stats.total_releases_found = release_count;
        self.stats.elements_processed = elements_processed;
        self.stats.bytes_processed = xml_reader.buffer_position() as usize;
        self.stats.parse_duration = start_time.elapsed();

        Ok(release_count)
    }

    /// Parse multiple releases with full content extraction
    pub fn parse_releases<R: BufRead>(
        &mut self,
        reader: R,
    ) -> Result<MultiReleaseResult, ParseError> {
        let start_time = std::time::Instant::now();
        let mut xml_reader = Reader::from_reader(crate::xml_text::Checked::new(reader));
        xml_reader.config_mut().trim_text(true);
        xml_reader.config_mut().check_end_names = true;

        let mut releases = Vec::new();
        let mut buf = Vec::new();
        let mut current_context: Option<ReleaseContext> = None;
        let mut depth = 0;
        let mut elements_processed = 0;
        let mut release_references = Vec::new();
        let message_header: Option<MessageHeader> = None;
        let mut in_release_list = false;
        let mut release_list_count = 0;

        let mut text_run = crate::xml_text::TextRun::default();
        loop {
            let event = xml_reader.read_event_into(&mut buf);
            crate::xml_text::check(xml_reader.get_ref())?;
            // One text value per run of Text / GeneralRef events (one Text event before
            // quick-xml 0.38), handled before the event that ends the run, as before.
            if let Ok(ref ev) = event {
                if text_run.push(ev) {
                    buf.clear();
                    continue;
                }
            }
            if let Some(raw) = text_run.take() {
                if let Some(ref mut context) = current_context {
                    // Use utf8_utils for proper UTF-8 handling
                    let current_pos = xml_reader.buffer_position() as usize;
                    let text = crate::utf8_utils::handle_escaped_text(&raw, current_pos)?
                        .trim()
                        .to_string();

                    if !text.is_empty() {
                        self.process_release_text_content(context, &text)?;
                    }
                }
            }
            match event {
                Ok(Event::Start(ref e)) => {
                    elements_processed += 1;
                    depth += 1;

                    // Security check
                    if depth > self.security_config.max_element_depth {
                        return Err(ParseError::DepthLimitExceeded {
                            depth,
                            limit: self.security_config.max_element_depth,
                        });
                    }

                    let element_name = self.extract_element_name(e.name().as_ref())?;

                    // Extract attributes
                    let mut attributes = HashMap::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        attributes.insert(key, value);
                    }

                    match element_name.as_str() {
                        "ReleaseList" | "ern:ReleaseList" => {
                            in_release_list = true;
                            release_list_count += 1;
                        }
                        "Release" | "ern:Release" if in_release_list => {
                            // Start new release
                            let is_main = attributes
                                .get("IsMainRelease")
                                .or_else(|| attributes.get("isMainRelease"))
                                .map(|v| v.to_lowercase() == "true");

                            let release = self.create_default_release();
                            current_context = Some(ReleaseContext {
                                release,
                                depth,
                                current_element_path: vec![element_name.clone()],
                                attributes: attributes.clone(),
                                is_main_release: is_main,
                                position: xml_reader.buffer_position() as usize,
                            });

                            if is_main.unwrap_or(false) {
                                self.stats.main_releases += 1;
                            } else {
                                self.stats.secondary_releases += 1;
                            }
                        }
                        _ => {
                            // Handle elements within release context
                            if let Some(ref mut context) = current_context {
                                context.current_element_path.push(element_name.clone());
                                self.process_release_element(
                                    context,
                                    &element_name,
                                    &attributes,
                                    &mut xml_reader,
                                    &mut buf,
                                )?;
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    depth = depth.saturating_sub(1);
                    let element_name = self.extract_element_name(e.name().as_ref())?;

                    match element_name.as_str() {
                        "ReleaseList" | "ern:ReleaseList" => {
                            in_release_list = false;
                        }
                        "Release" | "ern:Release" => {
                            if let Some(context) = current_context.take() {
                                // Extract release reference if available
                                if let Some(reference) =
                                    self.extract_release_reference(&context.release)
                                {
                                    release_references.push(reference);
                                }

                                releases.push(context.release);
                                self.stats.releases_parsed += 1;

                                // Check if we've reached the maximum
                                if self.max_releases > 0 && releases.len() >= self.max_releases {
                                    break;
                                }
                            }
                        }
                        _ => {
                            if let Some(ref mut context) = current_context {
                                context.current_element_path.pop();
                            }
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    elements_processed += 1;
                    let element_name = self.extract_element_name(e.name().as_ref())?;

                    // Handle empty Release elements (rare but possible)
                    if (element_name == "Release" || element_name.ends_with(":Release"))
                        && in_release_list
                    {
                        let mut attributes = HashMap::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            attributes.insert(key, value);
                        }

                        let is_main = attributes
                            .get("IsMainRelease")
                            .or_else(|| attributes.get("isMainRelease"))
                            .map(|v| v.to_lowercase() == "true");

                        let release = self.create_default_release();
                        releases.push(release);

                        if is_main.unwrap_or(false) {
                            self.stats.main_releases += 1;
                        } else {
                            self.stats.secondary_releases += 1;
                        }

                        self.stats.releases_parsed += 1;

                        if self.max_releases > 0 && releases.len() >= self.max_releases {
                            break;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error: {}", e)));
                }
                _ => {} // Skip other events
            }
            buf.clear();
        }

        // Finalize statistics
        self.stats.total_releases_found = self.stats.releases_parsed;
        self.stats.elements_processed = elements_processed;
        self.stats.bytes_processed = xml_reader.buffer_position() as usize;
        self.stats.parse_duration = start_time.elapsed();
        self.stats.release_list_count = release_list_count;

        Ok(MultiReleaseResult {
            releases,
            stats: self.stats.clone(),
            message_header,
            release_count: self.stats.releases_parsed,
            release_references,
        })
    }

    /// Extract element name, handling namespaces
    fn extract_element_name(&self, qname: &[u8]) -> Result<String, ParseError> {
        let name_str = std::str::from_utf8(qname)
            .map_err(|_| ParseError::IoError("Invalid UTF-8 in element name".to_string()))?;
        Ok(name_str.to_string())
    }

    /// Create a default release structure
    fn create_default_release(&self) -> Release {
        Release {
            // Filled from the document; absent values stay empty (before 0.5.1: an invented
            // reference with a timestamp and the title "Untitled Release").
            release_reference: String::new(),
            release_id: Vec::new(),
            release_title: Vec::new(),
            release_subtitle: None,
            release_type: None,
            genre: Vec::new(),
            release_resource_reference_list: Vec::new(),
            display_artist: Vec::new(),
            party_list: Vec::new(),
            release_date: Vec::new(),
            territory_code: Vec::new(),
            excluded_territory_code: Vec::new(),
            extensions: None,
            attributes: None,
            comments: None,
            display_title_text: Vec::new(),
            display_title: Vec::new(),
            display_artist_name: Vec::new(),
            resource_group: None,
            release_type_user_defined_value: None,
            parental_warning_type: Vec::new(),
            release_label_reference: Vec::new(),
            c_line: Vec::new(),
            label_name: Vec::new(),
            original_release_date: Vec::new(),
            p_line: Vec::new(),
            parental_warnings: Vec::new(),
        }
    }

    /// Process elements within a release context
    fn process_release_element(
        &self,
        context: &mut ReleaseContext,
        element_name: &str,
        attributes: &HashMap<String, String>,
        _reader: &mut Reader<impl BufRead>,
        _buf: &mut [u8],
    ) -> Result<(), ParseError> {
        // Update the release based on the element
        match element_name {
            "ReleaseReference" | "ern:ReleaseReference" => {
                // Will be filled by text content
            }
            "ReleaseId" | "ern:ReleaseId" => {
                // Will be filled by text content with namespace info
            }
            "ReferenceTitle" | "ern:ReferenceTitle" => {
                // Start of title section
            }
            "TitleText" | "ern:TitleText" => {
                // Will be filled by text content
            }
            "ReleaseType" | "ern:ReleaseType" => {
                // Will be filled by text content
            }
            _ => {
                // Handle other elements as needed
            }
        }

        // Store attributes for potential use
        for (key, value) in attributes {
            context
                .attributes
                .insert(format!("{}:{}", element_name, key), value.clone());
        }

        Ok(())
    }

    /// Process text content within a release
    fn process_release_text_content(
        &self,
        context: &mut ReleaseContext,
        text: &str,
    ) -> Result<(), ParseError> {
        let current_path = context.current_element_path.join("/");

        if current_path.contains("ReleaseReference") {
            context.release.release_reference = text.to_string();
        } else if current_path.contains("ReleaseId") {
            // Add to release ID list
            context.release.release_id.push(Identifier {
                id_type: IdentifierType::Proprietary,
                namespace: None,
                value: text.to_string(),
            });
        } else if current_path.contains("TitleText") {
            // Update the title
            if !context.release.release_title.is_empty() {
                context.release.release_title[0] = LocalizedString::new(text.to_string());
            } else {
                context
                    .release
                    .release_title
                    .push(LocalizedString::new(text.to_string()));
            }
        } else if current_path.contains("ReleaseType") {
            context.release.release_type = Some(match text {
                "Album" => ReleaseType::Album,
                "Single" => ReleaseType::Single,
                "EP" => ReleaseType::EP,
                "Compilation" => ReleaseType::Compilation,
                other => ReleaseType::Other(other.to_string()),
            });
        }

        Ok(())
    }

    /// Extract release reference from a release
    fn extract_release_reference(&self, release: &Release) -> Option<String> {
        if !release.release_reference.is_empty() {
            Some(release.release_reference.clone())
        } else if !release.release_id.is_empty() {
            Some(release.release_id[0].value.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_release_counting() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:MessageHeader>
                <ern:MessageId>MSG001</ern:MessageId>
            </ern:MessageHeader>
            <ern:ReleaseList>
                <ern:Release IsMainRelease="true">
                    <ern:ReleaseReference>REL001</ern:ReleaseReference>
                    <ern:ReferenceTitle>
                        <ern:TitleText>Album One</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:Release>
                <ern:Release IsMainRelease="false">
                    <ern:ReleaseReference>REL002</ern:ReleaseReference>
                    <ern:ReferenceTitle>
                        <ern:TitleText>Album Two</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:Release>
                <ern:Release>
                    <ern:ReleaseReference>REL003</ern:ReleaseReference>
                    <ern:ReferenceTitle>
                        <ern:TitleText>Album Three</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:Release>
            </ern:ReleaseList>
        </ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let mut parser = MultiReleaseParser::new(ERNVersion::V4_3);

        let count = parser
            .count_releases(cursor)
            .expect("Should count releases");

        assert_eq!(count, 3);
        assert_eq!(parser.stats.total_releases_found, 3);
        assert!(parser.stats.elements_processed > 0);
        assert!(parser.stats.bytes_processed > 0);
    }

    #[test]
    fn test_multi_release_parsing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:MessageHeader>
                <ern:MessageId>MSG001</ern:MessageId>
                <ern:MessageSender>
                    <ern:PartyName>Test Label</ern:PartyName>
                </ern:MessageSender>
                <ern:MessageRecipient>
                    <ern:PartyName>Test Recipient</ern:PartyName>
                </ern:MessageRecipient>
                <ern:MessageCreatedDateTime>2024-01-15T10:30:00Z</ern:MessageCreatedDateTime>
            </ern:MessageHeader>
            <ern:ReleaseList>
                <ern:Release IsMainRelease="true">
                    <ern:ReleaseReference>MAIN_RELEASE_001</ern:ReleaseReference>
                    <ern:ReleaseId Namespace="GRid">A1-123456789-1234567890-A</ern:ReleaseId>
                    <ern:ReferenceTitle>
                        <ern:TitleText>My Main Album</ern:TitleText>
                    </ern:ReferenceTitle>
                    <ern:ReleaseType>Album</ern:ReleaseType>
                </ern:Release>
                <ern:Release IsMainRelease="false">
                    <ern:ReleaseReference>SECONDARY_RELEASE_002</ern:ReleaseReference>
                    <ern:ReleaseId>REL_SEC_002</ern:ReleaseId>
                    <ern:ReferenceTitle>
                        <ern:TitleText>Bonus Tracks</ern:TitleText>
                    </ern:ReferenceTitle>
                    <ern:ReleaseType>EP</ern:ReleaseType>
                </ern:Release>
            </ern:ReleaseList>
        </ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let mut parser = MultiReleaseParser::new(ERNVersion::V4_3).detailed_parsing(true);

        let result = parser
            .parse_releases(cursor)
            .expect("Should parse releases");

        assert_eq!(result.releases.len(), 2);
        assert_eq!(result.release_count, 2);
        assert_eq!(result.stats.main_releases, 1);
        assert_eq!(result.stats.secondary_releases, 1);

        // Check parsed content
        let main_release = &result.releases[0];
        assert_eq!(main_release.release_reference, "MAIN_RELEASE_001");
        assert_eq!(main_release.release_title[0].text, "My Main Album");
        assert_eq!(
            main_release.release_type.as_ref().unwrap(),
            &ReleaseType::Album
        );

        let secondary_release = &result.releases[1];
        assert_eq!(secondary_release.release_reference, "SECONDARY_RELEASE_002");
        assert_eq!(secondary_release.release_title[0].text, "Bonus Tracks");
        assert_eq!(
            secondary_release.release_type.as_ref().unwrap(),
            &ReleaseType::EP
        );

        // Check references were extracted
        assert_eq!(result.release_references.len(), 2);
        assert!(result
            .release_references
            .contains(&"MAIN_RELEASE_001".to_string()));
        assert!(result
            .release_references
            .contains(&"SECONDARY_RELEASE_002".to_string()));

        println!("Multi-release parsing stats: {:#?}", result.stats);
    }

    #[test]
    fn test_max_releases_limit() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:ReleaseList>
                <ern:Release><ern:ReleaseReference>REL001</ern:ReleaseReference></ern:Release>
                <ern:Release><ern:ReleaseReference>REL002</ern:ReleaseReference></ern:Release>
                <ern:Release><ern:ReleaseReference>REL003</ern:ReleaseReference></ern:Release>
                <ern:Release><ern:ReleaseReference>REL004</ern:ReleaseReference></ern:Release>
                <ern:Release><ern:ReleaseReference>REL005</ern:ReleaseReference></ern:Release>
            </ern:ReleaseList>
        </ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let mut parser = MultiReleaseParser::new(ERNVersion::V4_3).max_releases(3);

        let result = parser
            .parse_releases(cursor)
            .expect("Should parse with limit");

        assert_eq!(result.releases.len(), 3);
        assert_eq!(result.release_count, 3);
        assert_eq!(result.stats.releases_parsed, 3);
    }

    #[test]
    fn test_empty_and_self_closing_releases() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:ReleaseList>
                <ern:Release/>
                <ern:Release IsMainRelease="true"/>
                <ern:Release>
                    <ern:ReleaseReference>REL003</ern:ReleaseReference>
                </ern:Release>
            </ern:ReleaseList>
        </ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let mut parser = MultiReleaseParser::new(ERNVersion::V4_3);

        let result = parser
            .parse_releases(cursor)
            .expect("Should parse empty releases");

        assert_eq!(result.releases.len(), 3);
        assert_eq!(result.stats.main_releases, 1);
        assert_eq!(result.stats.secondary_releases, 2);
    }

    #[test]
    fn test_performance_with_many_releases() {
        // Generate XML with many releases
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:ReleaseList>"#,
        );

        for i in 0..1000 {
            xml.push_str(&format!(
                r#"
                <ern:Release IsMainRelease="{}">
                    <ern:ReleaseReference>REL{:06}</ern:ReleaseReference>
                    <ern:ReferenceTitle>
                        <ern:TitleText>Release {}</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:Release>"#,
                i == 0,
                i,
                i
            ));
        }
        xml.push_str("</ern:ReleaseList></ern:NewReleaseMessage>");

        let cursor = Cursor::new(xml.as_bytes());
        let mut parser = MultiReleaseParser::new(ERNVersion::V4_3);

        let start = std::time::Instant::now();
        let count = parser
            .count_releases(cursor)
            .expect("Should count many releases");
        let count_duration = start.elapsed();

        assert_eq!(count, 1000);

        // Test parsing performance
        let cursor2 = Cursor::new(xml.as_bytes());
        let mut parser2 = MultiReleaseParser::new(ERNVersion::V4_3)
            .detailed_parsing(true)
            .max_releases(100); // Limit for performance test

        let start2 = std::time::Instant::now();
        let result = parser2
            .parse_releases(cursor2)
            .expect("Should parse many releases");
        let parse_duration = start2.elapsed();

        assert_eq!(result.releases.len(), 100);
        assert_eq!(result.stats.main_releases, 1);
        assert_eq!(result.stats.secondary_releases, 99);

        println!("Performance test results:");
        println!("  Count 1000 releases: {:?}", count_duration);
        println!("  Parse 100 releases: {:?}", parse_duration);
        println!(
            "  Count throughput: {:.0} releases/sec",
            1000.0 / count_duration.as_secs_f64()
        );
        println!(
            "  Parse throughput: {:.0} releases/sec",
            100.0 / parse_duration.as_secs_f64()
        );
    }
}
