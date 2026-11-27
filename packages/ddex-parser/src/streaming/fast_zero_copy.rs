//! Experimental low-level streaming parser (`FastZeroCopyParser`)
//!
//! It reads the input in chunks, finds elements with memchr and interns strings to
//! limit allocations. Despite the name it copies element content. It yields raw elements,
//! not the release model.

use crate::error::ParseError;
use crate::streaming::{WorkingStreamingElement, WorkingStreamingStats};
use ddex_core::models::versions::ERNVersion;
use std::collections::HashMap;
use std::io::BufRead;
use std::time::Instant;

/// Experimental chunked streaming parser (copies element content despite its name)
pub struct FastZeroCopyParser {
    /// Buffer for reading chunks
    read_buffer: Vec<u8>,
    /// Leftover data from previous chunk
    leftover: Vec<u8>,
    /// Scans the chunks for references to undefined entities
    references: ddex_core::xml_refs::ReferenceScanner,
    /// String cache for avoiding allocations
    string_cache: HashMap<Vec<u8>, String>,
    /// Statistics
    bytes_processed: u64,
    elements_found: u64,
    start_time: Instant,
}

impl FastZeroCopyParser {
    pub fn new() -> Self {
        Self {
            read_buffer: vec![0; 64 * 1024], // 64KB buffer
            leftover: Vec::new(),
            references: ddex_core::xml_refs::ReferenceScanner::rejecting_doctype(),
            string_cache: HashMap::with_capacity(512),
            bytes_processed: 0,
            elements_found: 0,
            start_time: Instant::now(),
        }
    }

    /// Get interned string to avoid allocations
    fn intern_string(&mut self, bytes: &[u8]) -> String {
        if let Some(cached) = self.string_cache.get(bytes) {
            cached.clone()
        } else {
            let s = String::from_utf8_lossy(bytes).to_string();
            self.string_cache.insert(bytes.to_vec(), s.clone());
            s
        }
    }

    /// Fast element extraction using memchr for initial scanning
    pub fn parse_chunk(
        &mut self,
        chunk: &[u8],
    ) -> Result<Vec<WorkingStreamingElement>, ParseError> {
        self.references
            .feed(chunk)
            .map_err(|entity| crate::xml_text::undefined_entity(&entity))?;
        if let Some(doctype) = self.references.doctype() {
            return Err(crate::xml_text::doctype_error(doctype));
        }
        self.bytes_processed += chunk.len() as u64;
        let mut results = Vec::new();

        // Combine leftover with new chunk
        let mut data = Vec::with_capacity(self.leftover.len() + chunk.len());
        data.extend_from_slice(&self.leftover);
        data.extend_from_slice(chunk);

        // Find MessageHeader elements
        let mut pos = 0;
        while let Some(start) = self.find_pattern(&data[pos..], b"<MessageHeader") {
            let abs_start = pos + start;
            if let Some(element) = self.extract_message_header_fast(&data, abs_start)? {
                results.push(element);
                self.elements_found += 1;
            }
            pos = abs_start + 14; // Skip past "<MessageHeader"
        }

        // Find Release elements
        pos = 0;
        while let Some(start) = self.find_pattern(&data[pos..], b"<Release ") {
            let abs_start = pos + start;
            if let Some(element) = self.extract_release_fast(&data, abs_start)? {
                results.push(element);
                self.elements_found += 1;
            }
            pos = abs_start + 9; // Skip past "<Release "
        }

        // Find SoundRecording elements
        pos = 0;
        while let Some(start) = self.find_pattern(&data[pos..], b"<SoundRecording ") {
            let abs_start = pos + start;
            if let Some(element) = self.extract_sound_recording_fast(&data, abs_start)? {
                results.push(element);
                self.elements_found += 1;
            }
            pos = abs_start + 16; // Skip past "<SoundRecording "
        }

        // Store leftover data that might contain incomplete elements
        if data.len() > 2048 {
            // Keep last 2KB to handle elements spanning chunks
            self.leftover.clear();
            self.leftover.extend_from_slice(&data[data.len() - 2048..]);
        } else {
            self.leftover = data;
        }

        Ok(results)
    }

    /// Fast pattern finding using memchr
    fn find_pattern(&self, data: &[u8], pattern: &[u8]) -> Option<usize> {
        if pattern.is_empty() {
            return None;
        }

        // Use memchr to find first byte quickly, then verify full pattern
        let mut pos = 0;
        while let Some(first_byte_pos) = memchr::memchr(pattern[0], &data[pos..]) {
            let abs_pos = pos + first_byte_pos;

            if abs_pos + pattern.len() <= data.len()
                && &data[abs_pos..abs_pos + pattern.len()] == pattern
            {
                return Some(abs_pos);
            }

            pos = abs_pos + 1;
        }

        None
    }

    /// Fast message header extraction
    fn extract_message_header_fast(
        &mut self,
        data: &[u8],
        start: usize,
    ) -> Result<Option<WorkingStreamingElement>, ParseError> {
        // Find closing tag
        if let Some(end) = self.find_pattern(&data[start..], b"</MessageHeader>") {
            let header_data = &data[start..start + end + 16]; // Include closing tag

            // Extract MessageId quickly
            let message_id = if let Some(id) = self.extract_tag_content(header_data, b"MessageId") {
                self.intern_string(id)
            } else {
                String::new()
            };

            // Absent values are empty (before 0.5.1: "unknown", the current time and ERN 4.3
            // whatever the document; the real element name was not read).
            let created_date_time = match self
                .extract_tag_content(header_data, b"MessageCreatedDateTime")
                .or_else(|| self.extract_tag_content(header_data, b"CreatedDateTime"))
            {
                Some(dt) => self.intern_string(dt),
                None => String::new(),
            };

            return Ok(Some(WorkingStreamingElement::MessageHeader {
                message_id,
                created_date_time,
                // The namespace's version; ERN 4.3 (what stream_zero_copy assumes) when the
                // chunk holds no ERN namespace.
                version: namespace_version(data).unwrap_or(ERNVersion::V4_3),
            }));
        }

        Ok(None)
    }

    /// Fast release extraction
    fn extract_release_fast(
        &mut self,
        data: &[u8],
        start: usize,
    ) -> Result<Option<WorkingStreamingElement>, ParseError> {
        // Find closing tag
        if let Some(end) = self.find_pattern(&data[start..], b"</Release>") {
            let release_data = &data[start..start + end + 10]; // Include closing tag

            // Extract ReleaseReference attribute from opening tag
            let reference = if let Some(attr) =
                self.extract_attribute_fast(release_data, b"ReleaseReference")
            {
                self.intern_string(attr)
            } else {
                String::new()
            };

            // Extract title from TitleText nested in ReferenceTitle
            let title =
                if let Some(title_data) = self.extract_tag_content(release_data, b"TitleText") {
                    self.intern_string(title_data)
                } else {
                    String::new()
                };

            // Extract resource references (simplified)
            let resource_references = self.extract_resource_references_fast(release_data);

            return Ok(Some(WorkingStreamingElement::Release {
                reference,
                title,
                resource_references,
            }));
        }

        Ok(None)
    }

    /// Fast sound recording extraction
    fn extract_sound_recording_fast(
        &mut self,
        data: &[u8],
        start: usize,
    ) -> Result<Option<WorkingStreamingElement>, ParseError> {
        if let Some(end) = self.find_pattern(&data[start..], b"</SoundRecording>") {
            let recording_data = &data[start..start + end + 17]; // Include closing tag

            let reference = if let Some(attr) =
                self.extract_attribute_fast(recording_data, b"ResourceReference")
            {
                self.intern_string(attr)
            } else {
                String::new()
            };

            let title =
                if let Some(title_data) = self.extract_tag_content(recording_data, b"TitleText") {
                    self.intern_string(title_data)
                } else {
                    String::new()
                };

            let duration = self
                .extract_tag_content(recording_data, b"Duration")
                .map(|d| self.intern_string(d));

            let isrc = self
                .extract_tag_content(recording_data, b"ISRC")
                .map(|i| self.intern_string(i));

            return Ok(Some(WorkingStreamingElement::SoundRecording {
                reference,
                title,
                duration,
                isrc,
            }));
        }

        Ok(None)
    }

    /// Extract content between XML tags
    fn extract_tag_content<'a>(&self, data: &'a [u8], tag_name: &[u8]) -> Option<&'a [u8]> {
        // Create opening and closing tags
        let opening = [b"<", tag_name, b">"].concat();
        let closing = [b"</", tag_name, b">"].concat();

        if let Some(start_pos) = self.find_pattern(data, &opening) {
            let content_start = start_pos + opening.len();
            if let Some(end_pos) = self.find_pattern(&data[content_start..], &closing) {
                let content_end = content_start + end_pos;
                return Some(&data[content_start..content_end]);
            }
        }

        None
    }

    /// Extract attribute value from XML tag
    fn extract_attribute_fast<'a>(&self, data: &'a [u8], attr_name: &[u8]) -> Option<&'a [u8]> {
        let pattern = [attr_name, b"=\""].concat();

        if let Some(start_pos) = self.find_pattern(data, &pattern) {
            let value_start = start_pos + pattern.len();

            // Find closing quote
            if let Some(quote_pos) = memchr::memchr(b'"', &data[value_start..]) {
                let value_end = value_start + quote_pos;
                return Some(&data[value_start..value_end]);
            }
        }

        None
    }

    /// Extract resource references (simplified)
    fn extract_resource_references_fast(&mut self, data: &[u8]) -> Vec<String> {
        let mut refs = Vec::new();
        let mut pos = 0;

        // Look for ResourceReference tags
        while let Some(start) = self.find_pattern(&data[pos..], b"<ResourceReference>") {
            let abs_start = pos + start;
            if let Some(content) =
                self.extract_tag_content(&data[abs_start..], b"ResourceReference")
            {
                refs.push(self.intern_string(content));
            }
            pos = abs_start + 19; // Skip past "<ResourceReference>"
        }

        refs
    }

    /// Get current statistics
    pub fn stats(&self) -> WorkingStreamingStats {
        let elapsed = self.start_time.elapsed();
        let throughput = if elapsed.as_secs_f64() > 0.0 {
            (self.bytes_processed as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64()
        } else {
            0.0
        };

        WorkingStreamingStats {
            bytes_processed: self.bytes_processed,
            elements_yielded: self.elements_found as usize,
            // Depth is not tracked by this scanner (before 0.6.0: 0, and a constant 10 as
            // the maximum).
            current_depth: None,
            current_memory_bytes: Some(self.read_buffer.capacity() + self.leftover.capacity()),
            max_memory_used_bytes: Some(self.read_buffer.capacity() + self.leftover.capacity()),
            elapsed_time: elapsed,
            throughput_mb_per_sec: throughput,
        }
    }
}

/// The ERN version named by an ERN namespace in `data`, if any.
fn namespace_version(data: &[u8]) -> Option<ERNVersion> {
    let find = |ns: &[u8]| memchr::memmem::find(data, ns).is_some();
    if find(b"http://ddex.net/xml/ern/43") {
        Some(ERNVersion::V4_3)
    } else if find(b"http://ddex.net/xml/ern/42") {
        Some(ERNVersion::V4_2)
    } else if find(b"http://ddex.net/xml/ern/382") {
        Some(ERNVersion::V3_8_2)
    } else if find(b"http://ddex.net/xml/ern/381") {
        Some(ERNVersion::V3_8_1)
    } else {
        None
    }
}

impl Default for FastZeroCopyParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Fast streaming iterator
pub struct FastZeroCopyIterator<R: BufRead> {
    reader: R,
    parser: FastZeroCopyParser,
    buffer: Vec<u8>,
    finished: bool,
    elements_queue: Vec<WorkingStreamingElement>,
    current_index: usize,
}

impl<R: BufRead> FastZeroCopyIterator<R> {
    pub fn new(reader: R, _version: ERNVersion) -> Self {
        Self {
            reader,
            parser: FastZeroCopyParser::new(),
            buffer: vec![0; 64 * 1024], // 64KB chunks
            finished: false,
            elements_queue: Vec::new(),
            current_index: 0,
        }
    }

    pub fn stats(&self) -> WorkingStreamingStats {
        self.parser.stats()
    }

    fn read_next_chunk(&mut self) -> Result<bool, ParseError> {
        let bytes_read = self.reader.read(&mut self.buffer)?;

        if bytes_read == 0 {
            return Ok(false); // EOF
        }

        let elements = self.parser.parse_chunk(&self.buffer[..bytes_read])?;
        self.elements_queue.extend(elements);

        Ok(true)
    }
}

impl<R: BufRead> Iterator for FastZeroCopyIterator<R> {
    type Item = Result<WorkingStreamingElement, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // Return queued elements first
        if self.current_index < self.elements_queue.len() {
            let element = self.elements_queue[self.current_index].clone();
            self.current_index += 1;
            return Some(Ok(element));
        }

        // Try to read more data
        match self.read_next_chunk() {
            Ok(true) => {
                // We read some data, try again
                self.next()
            }
            Ok(false) => {
                // EOF reached
                self.finished = true;
                Some(Ok(WorkingStreamingElement::EndOfStream {
                    stats: self.parser.stats(),
                }))
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_fast_zero_copy_basic() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>FAST-TEST-MSG</MessageId>
        <CreatedDateTime>2023-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>
    <Release ReleaseReference="FAST-REL-001">
        <ReferenceTitle>
            <TitleText>Fast Zero Copy Release</TitleText>
        </ReferenceTitle>
    </Release>
</ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let iterator = FastZeroCopyIterator::new(cursor, ERNVersion::V4_3);

        let elements: Result<Vec<_>, _> = iterator.collect();
        assert!(elements.is_ok(), "Fast zero-copy parsing should work");

        let elements = elements.unwrap();
        assert!(!elements.is_empty(), "Should find elements");

        // Verify elements
        let has_header = elements
            .iter()
            .any(|e| matches!(e, WorkingStreamingElement::MessageHeader { .. }));
        let has_release = elements
            .iter()
            .any(|e| matches!(e, WorkingStreamingElement::Release { .. }));
        let has_end_stream = elements
            .iter()
            .any(|e| matches!(e, WorkingStreamingElement::EndOfStream { .. }));

        assert!(has_header, "Should find message header");
        assert!(has_release, "Should find release");
        assert!(has_end_stream, "Should find end of stream");

        println!("✅ Fast zero-copy parser basic test passed!");
    }

    #[test]
    fn test_fast_pattern_matching() {
        let parser = FastZeroCopyParser::new();
        let data = b"<Release><MessageHeader><SoundRecording>";

        assert_eq!(parser.find_pattern(data, b"<Release>"), Some(0));
        assert_eq!(parser.find_pattern(data, b"<MessageHeader>"), Some(9));
        assert_eq!(parser.find_pattern(data, b"<SoundRecording>"), Some(24));
        assert_eq!(parser.find_pattern(data, b"<NotFound>"), None);
    }

    #[test]
    fn test_tag_content_extraction() {
        let parser = FastZeroCopyParser::new();
        let data = b"<Title>Test Title</Title>";

        let content = parser.extract_tag_content(data, b"Title").unwrap();
        assert_eq!(content, b"Test Title");
    }

    #[test]
    fn test_attribute_extraction() {
        let parser = FastZeroCopyParser::new();
        let data = b"<Release ReleaseReference=\"REL-123\">";

        let attr_value = parser
            .extract_attribute_fast(data, b"ReleaseReference")
            .unwrap();
        assert_eq!(attr_value, b"REL-123");
    }
}
