// core/src/parser/stream.rs
//! Streaming parser for large DDEX files

use crate::error::ParseError;
use crate::parser::ParseOptions;
use crate::transform::flatten::Flattener;
use crate::utf8_utils;
use ddex_core::models::flat::ParsedERNMessage;
use ddex_core::models::graph::{Deal, ERNMessage, MessageHeader, Party, Release, Resource};
use ddex_core::models::versions::ERNVersion;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::BufRead;
use std::time::{Duration, Instant};

/// Progress information for streaming parsing
#[derive(Debug, Clone)]
pub struct ParseProgress {
    pub bytes_processed: u64,
    pub releases_parsed: usize,
    pub resources_parsed: usize,
    pub elapsed: Duration,
    pub estimated_total_bytes: Option<u64>,
}

/// Streaming parser for memory-efficient processing
///
/// Part of the public streaming API for parsing large DDEX files efficiently.
#[allow(dead_code)]
pub struct StreamingParser<R: BufRead> {
    reader: Reader<R>,
    _version: ERNVersion,
    progress_callback: Option<Box<dyn FnMut(ParseProgress) + Send>>,
    start_time: Instant,
    bytes_processed: u64,
    releases_parsed: usize,
    resources_parsed: usize,
    chunk_size: usize,
    max_memory: usize,
    buffer: Vec<u8>,
    current_depth: usize,
    max_depth: usize,
}

impl<R: BufRead> StreamingParser<R> {
    pub fn new(reader: R, version: ERNVersion) -> Self {
        Self::new_with_security_config(
            reader,
            version,
            &crate::parser::security::SecurityConfig::default(),
        )
    }

    pub fn new_with_security_config(
        reader: R,
        version: ERNVersion,
        security_config: &crate::parser::security::SecurityConfig,
    ) -> Self {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        xml_reader.config_mut().check_end_names = true;
        xml_reader.config_mut().expand_empty_elements = false;

        Self {
            reader: xml_reader,
            _version: version,
            progress_callback: None,
            start_time: Instant::now(),
            bytes_processed: 0,
            releases_parsed: 0,
            resources_parsed: 0,
            chunk_size: 100,
            max_memory: 100 * 1024 * 1024, // 100MB default
            buffer: Vec::with_capacity(8192),
            current_depth: 0,
            max_depth: security_config.max_element_depth,
        }
    }

    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: FnMut(ParseProgress) + Send + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn with_max_memory(mut self, max: usize) -> Self {
        self.max_memory = max;
        self
    }

    fn update_progress(&mut self) {
        if let Some(ref mut callback) = self.progress_callback {
            let progress = ParseProgress {
                bytes_processed: self.bytes_processed,
                releases_parsed: self.releases_parsed,
                resources_parsed: self.resources_parsed,
                elapsed: self.start_time.elapsed(),
                estimated_total_bytes: None,
            };
            callback(progress);
        }
    }

    fn update_byte_position(&mut self) {
        self.bytes_processed = self.reader.buffer_position();
    }

    /// Parse the message header
    pub fn parse_header(&mut self) -> Result<MessageHeader, ParseError> {
        self.buffer.clear();

        // Skip to MessageHeader element
        loop {
            match self.reader.read_event_into(&mut self.buffer) {
                Ok(Event::Start(ref e)) => {
                    self.current_depth += 1;

                    // Check depth limit
                    if self.current_depth > self.max_depth {
                        return Err(ParseError::DepthLimitExceeded {
                            depth: self.current_depth,
                            limit: self.max_depth,
                        });
                    }

                    if e.name().as_ref() == b"MessageHeader" {
                        return self.parse_message_header_element();
                    } else if self.current_depth > 1 {
                        self.skip_element()?;
                    }
                    // Depth 1 is the message root (e.g. NewReleaseMessage): descend into it
                    // rather than skipping the whole document.
                }
                Ok(Event::End(_)) => {
                    self.current_depth = self.current_depth.saturating_sub(1);
                }
                Ok(Event::Eof) => {
                    return Err(ParseError::XmlError("No MessageHeader found".to_string()));
                }
                Err(e) => {
                    return Err(ParseError::XmlError(e.to_string()));
                }
                _ => {}
            }
            self.buffer.clear();
        }
    }

    fn parse_message_header_element(&mut self) -> Result<MessageHeader, ParseError> {
        use ddex_core::models::graph::{MessageRecipient, MessageSender, MessageType};

        let mut message_id = String::new();
        let message_type = MessageType::NewReleaseMessage;
        let mut created_date_time = chrono::Utc::now();
        let mut sender = MessageSender {
            party_id: Vec::new(),
            party_name: Vec::new(),
            trading_name: None,
            extensions: None,
            attributes: None,
            comments: None,
        };
        let mut recipient = MessageRecipient {
            party_id: Vec::new(),
            party_name: Vec::new(),
            trading_name: None,
            extensions: None,
            attributes: None,
            comments: None,
        };

        self.buffer.clear();
        loop {
            match self.reader.read_event_into(&mut self.buffer) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"MessageId" => {
                        message_id = self.read_text_element()?;
                    }
                    b"MessageCreatedDateTime" => {
                        let text = self.read_text_element()?;
                        created_date_time = chrono::DateTime::parse_from_rfc3339(&text)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| chrono::Utc::now());
                    }
                    b"MessageSender" => {
                        sender = self.parse_message_sender()?;
                    }
                    b"MessageRecipient" => {
                        recipient = self.parse_message_recipient()?;
                    }
                    _ => {
                        self.skip_element()?;
                    }
                },
                Ok(Event::End(ref e)) if e.name().as_ref() == b"MessageHeader" => {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(ParseError::XmlError(
                        "Unexpected EOF in MessageHeader".to_string(),
                    ));
                }
                Err(e) => {
                    return Err(ParseError::XmlError(format!(
                        "XML error at {}: {}",
                        self.get_current_location(),
                        e
                    )));
                }
                _ => {}
            }
            self.buffer.clear();
        }

        Ok(MessageHeader {
            message_id,
            message_type,
            message_created_date_time: created_date_time,
            message_sender: sender,
            message_recipient: recipient,
            message_control_type: None,
            message_thread_id: None,
            extensions: None,
            attributes: None,
            comments: None,
        })
    }

    fn parse_message_sender(
        &mut self,
    ) -> Result<ddex_core::models::graph::MessageSender, ParseError> {
        use ddex_core::models::common::{Identifier, LocalizedString};

        let mut sender = ddex_core::models::graph::MessageSender {
            party_id: Vec::new(),
            party_name: Vec::new(),
            trading_name: None,
            extensions: None,
            attributes: None,
            comments: None,
        };

        self.buffer.clear();
        loop {
            match self.reader.read_event_into(&mut self.buffer) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"PartyId" => {
                        let value = self.read_text_element()?;
                        sender.party_id.push(Identifier {
                            id_type: ddex_core::models::common::IdentifierType::Proprietary,
                            namespace: None,
                            value,
                        });
                    }
                    b"PartyName" => {
                        let text = self.read_text_element()?;
                        sender.party_name.push(LocalizedString::new(text));
                    }
                    _ => {
                        self.skip_element()?;
                    }
                },
                Ok(Event::End(ref e)) if e.name().as_ref() == b"MessageSender" => {
                    break;
                }
                _ => {}
            }
            self.buffer.clear();
        }

        Ok(sender)
    }

    fn parse_message_recipient(
        &mut self,
    ) -> Result<ddex_core::models::graph::MessageRecipient, ParseError> {
        // Similar to parse_message_sender
        use ddex_core::models::common::{Identifier, LocalizedString};

        let mut recipient = ddex_core::models::graph::MessageRecipient {
            party_id: Vec::new(),
            party_name: Vec::new(),
            trading_name: None,
            extensions: None,
            attributes: None,
            comments: None,
        };

        self.buffer.clear();
        loop {
            match self.reader.read_event_into(&mut self.buffer) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"PartyId" => {
                        let value = self.read_text_element()?;
                        recipient.party_id.push(Identifier {
                            id_type: ddex_core::models::common::IdentifierType::Proprietary,
                            namespace: None,
                            value,
                        });
                    }
                    b"PartyName" => {
                        let text = self.read_text_element()?;
                        recipient.party_name.push(LocalizedString::new(text));
                    }
                    _ => {
                        self.skip_element()?;
                    }
                },
                Ok(Event::End(ref e)) if e.name().as_ref() == b"MessageRecipient" => {
                    break;
                }
                _ => {}
            }
            self.buffer.clear();
        }

        Ok(recipient)
    }

    /// Stream releases one at a time for memory efficiency
    pub fn stream_releases(&mut self) -> ReleaseIterator<'_, R> {
        ReleaseIterator::new(self)
    }

    /// Stream resources one at a time
    pub fn stream_resources(&mut self) -> ResourceIterator<'_, R> {
        ResourceIterator::new(self)
    }

    /// Stream parties
    pub fn stream_parties(&mut self) -> PartyIterator<'_, R> {
        PartyIterator::new(self)
    }

    /// Stream deals
    pub fn stream_deals(&mut self) -> DealIterator<'_, R> {
        DealIterator::new(self)
    }

    /// Helper to read text content of current element
    fn read_text_element(&mut self) -> Result<String, ParseError> {
        let mut text = String::new();
        self.buffer.clear();

        loop {
            let event = self.reader.read_event_into(&mut self.buffer);
            match event {
                Ok(Event::Text(e)) => {
                    // Use proper UTF-8 handling from utf8_utils
                    let current_pos = self.reader.buffer_position() as usize;
                    text = utf8_utils::handle_text_node(&e, current_pos)?;
                }
                Ok(Event::End(_)) => {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(ParseError::XmlError("Unexpected EOF".to_string()));
                }
                Err(e) => {
                    let location = self.get_current_location();
                    return Err(ParseError::XmlError(format!(
                        "XML error at {}: {}",
                        location, e
                    )));
                }
                _ => {}
            }
            self.buffer.clear();
        }

        Ok(text)
    }

    /// Skip an element and all its children
    fn skip_element(&mut self) -> Result<(), ParseError> {
        let mut local_depth = 1;
        self.buffer.clear();

        while local_depth > 0 {
            match self.reader.read_event_into(&mut self.buffer) {
                Ok(Event::Start(_)) => {
                    local_depth += 1;
                    self.current_depth += 1;

                    // Check depth limit
                    if self.current_depth > self.max_depth {
                        return Err(ParseError::DepthLimitExceeded {
                            depth: self.current_depth,
                            limit: self.max_depth,
                        });
                    }
                }
                Ok(Event::End(_)) => {
                    local_depth -= 1;
                    self.current_depth = self.current_depth.saturating_sub(1);
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ParseError::XmlError(e.to_string()));
                }
                _ => {}
            }
            self.buffer.clear();
        }

        Ok(())
    }

    fn get_current_location(&self) -> String {
        format!(
            "byte offset {} in /NewReleaseMessage",
            self.reader.buffer_position()
        )
    }
}

/// Iterator for streaming releases
///
/// Part of the public streaming API.
#[allow(dead_code)]
pub struct ReleaseIterator<'a, R: BufRead> {
    parser: &'a mut StreamingParser<R>,
    done: bool,
    in_release_list: bool,
}

impl<'a, R: BufRead> ReleaseIterator<'a, R> {
    fn new(parser: &'a mut StreamingParser<R>) -> Self {
        Self {
            parser,
            done: false,
            in_release_list: false,
        }
    }

    fn find_next_release(&mut self) -> Result<Option<Release>, ParseError> {
        loop {
            self.parser.buffer.clear();
            match self.parser.reader.read_event_into(&mut self.parser.buffer) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"ReleaseList" => {
                        self.in_release_list = true;
                    }
                    b"Release" if self.in_release_list => {
                        return self.parse_release_element();
                    }
                    _ => {
                        self.parser.skip_element()?;
                    }
                },
                Ok(Event::End(ref e)) if e.name().as_ref() == b"ReleaseList" => {
                    self.done = true;
                    return Ok(None);
                }
                Ok(Event::Eof) => {
                    self.done = true;
                    return Ok(None);
                }
                Err(e) => {
                    return Err(ParseError::XmlError(e.to_string()));
                }
                _ => {}
            }
        }
    }

    fn parse_release_element(&mut self) -> Result<Option<Release>, ParseError> {
        use ddex_core::models::common::LocalizedString;

        let mut release = Release {
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
        };

        self.parser.buffer.clear();
        loop {
            match self.parser.reader.read_event_into(&mut self.parser.buffer) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"ReleaseReference" => {
                        release.release_reference = self.parser.read_text_element()?;
                    }
                    b"ReferenceTitle" | b"Title" => {
                        let text = self.parser.read_text_element()?;
                        release.release_title.push(LocalizedString::new(text));
                    }
                    _ => {
                        self.parser.skip_element()?;
                    }
                },
                Ok(Event::End(ref e)) if e.name().as_ref() == b"Release" => {
                    break;
                }
                _ => {}
            }
            self.parser.buffer.clear();
        }

        self.parser.releases_parsed += 1;
        self.parser.update_byte_position();
        self.parser.update_progress();

        // Check memory limit
        let estimated_size = std::mem::size_of::<Release>() * self.parser.releases_parsed;
        if estimated_size > self.parser.max_memory {
            return Err(ParseError::SecurityViolation {
                message: format!(
                    "Memory limit exceeded: {} > {}",
                    estimated_size, self.parser.max_memory
                ),
            });
        }

        // Yield control periodically
        if self.parser.releases_parsed % self.parser.chunk_size == 0 {
            std::thread::yield_now();
        }

        Ok(Some(release))
    }
}

impl<'a, R: BufRead> Iterator for ReleaseIterator<'a, R> {
    type Item = Result<Release, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.find_next_release() {
            Ok(Some(release)) => Some(Ok(release)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

// Similar iterators for other types
pub struct ResourceIterator<'a, R: BufRead> {
    _parser: &'a mut StreamingParser<R>,
    _done: bool,
    _in_resource_list: bool,
}

impl<'a, R: BufRead> ResourceIterator<'a, R> {
    fn new(parser: &'a mut StreamingParser<R>) -> Self {
        Self {
            _parser: parser,
            _done: false,
            _in_resource_list: false,
        }
    }
}

impl<'a, R: BufRead> Iterator for ResourceIterator<'a, R> {
    type Item = Result<Resource, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Similar implementation to ReleaseIterator
        None // Placeholder
    }
}

pub struct PartyIterator<'a, R: BufRead> {
    _parser: &'a mut StreamingParser<R>,
    _done: bool,
}

impl<'a, R: BufRead> PartyIterator<'a, R> {
    fn new(parser: &'a mut StreamingParser<R>) -> Self {
        Self {
            _parser: parser,
            _done: false,
        }
    }
}

impl<'a, R: BufRead> Iterator for PartyIterator<'a, R> {
    type Item = Result<Party, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        None // Placeholder
    }
}

/// Streams the deals of the `DealList`.
///
/// Each `ReleaseDeal` is parsed with the same logic as the DOM path, so every `Deal` carries
/// the `DealReleaseReference`s of its enclosing `ReleaseDeal`.
pub struct DealIterator<'a, R: BufRead> {
    parser: &'a mut StreamingParser<R>,
    done: bool,
    in_deal_list: bool,
    pending: std::collections::VecDeque<Deal>,
}

impl<'a, R: BufRead> DealIterator<'a, R> {
    fn new(parser: &'a mut StreamingParser<R>) -> Self {
        Self {
            parser,
            done: false,
            in_deal_list: false,
            pending: std::collections::VecDeque::new(),
        }
    }

    /// Read up to the next `ReleaseDeal` and parse it into its deals.
    fn read_next_release_deal(&mut self) -> Result<Option<Vec<Deal>>, ParseError> {
        loop {
            self.parser.buffer.clear();
            let release_deal_start =
                match self.parser.reader.read_event_into(&mut self.parser.buffer) {
                    Ok(Event::Start(ref e)) => match e.name().as_ref() {
                        b"DealList" => {
                            self.parser.current_depth += 1;
                            self.in_deal_list = true;
                            continue;
                        }
                        b"ReleaseDeal" if self.in_deal_list => e.to_owned(),
                        // Not yet inside the message root (stream_deals called at the start of the
                        // document): descend into the root instead of skipping it.
                        _ if self.parser.current_depth == 0 => {
                            self.parser.current_depth += 1;
                            continue;
                        }
                        _ => {
                            self.parser.current_depth += 1;
                            self.parser.skip_element()?;
                            continue;
                        }
                    },
                    Ok(Event::End(ref e)) if e.name().as_ref() == b"DealList" => {
                        self.parser.current_depth = self.parser.current_depth.saturating_sub(1);
                        return Ok(None);
                    }
                    Ok(Event::Eof) => return Ok(None),
                    Err(e) => return Err(ParseError::XmlError(e.to_string())),
                    _ => continue,
                };

            // The validator must see the ReleaseDeal start tag so the matching end tag balances.
            let mut validator = crate::parser::xml_validator::XmlValidator::strict();
            validator.validate_event(&Event::Start(release_deal_start), &self.parser.reader)?;
            let builder = crate::transform::graph::GraphBuilder::new(self.parser._version);
            return builder
                .parse_release_deal(&mut self.parser.reader, &mut validator)
                .map(Some);
        }
    }
}

impl<'a, R: BufRead> Iterator for DealIterator<'a, R> {
    type Item = Result<Deal, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(deal) = self.pending.pop_front() {
                return Some(Ok(deal));
            }
            if self.done {
                return None;
            }
            match self.read_next_release_deal() {
                Ok(Some(deals)) => self.pending.extend(deals),
                Ok(None) => self.done = true,
                Err(e) => {
                    self.done = true;
                    return Some(Err(e));
                }
            }
        }
    }
}

/// Parse using streaming for large files
pub fn parse_streaming<R: BufRead>(
    reader: R,
    version: ERNVersion,
    options: ParseOptions,
    security_config: &crate::parser::security::SecurityConfig,
) -> Result<ParsedERNMessage, ParseError> {
    let mut parser = StreamingParser::new_with_security_config(reader, version, security_config)
        .with_chunk_size(options.chunk_size)
        .with_max_memory(options.max_memory);

    // Parse header first
    let message_header = parser.parse_header()?;

    // Collect releases in chunks to limit memory
    let mut releases = Vec::new();
    let mut resources = Vec::new();
    let mut parties = Vec::new();
    let mut deals = Vec::new();

    // Stream releases
    for release_result in parser.stream_releases() {
        let release = release_result?;
        releases.push(release);
    }

    // Stream resources
    for resource_result in parser.stream_resources() {
        let resource = resource_result?;
        resources.push(resource);
    }

    // Stream parties
    for party_result in parser.stream_parties() {
        let party = party_result?;
        parties.push(party);
    }

    // Stream deals
    for deal_result in parser.stream_deals() {
        let deal = deal_result?;
        deals.push(deal);
    }

    // Build ERNMessage
    let graph = ERNMessage {
        message_header,
        parties,
        resources,
        releases,
        deals,
        version,
        profile: None,
        message_audit_trail: None,
        extensions: None,
        legacy_extensions: None,
        comments: None,
        attributes: None,
        track_releases: Vec::new(),
    };

    // Flatten to developer-friendly model
    let flat = Flattener::flatten(graph.clone());

    Ok(ParsedERNMessage {
        graph,
        flat: flat?,
        extensions: None,
    })
}
