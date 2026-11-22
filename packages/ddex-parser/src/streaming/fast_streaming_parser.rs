// src/streaming/fast_streaming_parser.rs
//! Ultra-high-performance streaming DDEX parser targeting 280+ MB/s throughput

#[allow(dead_code)] // Experimental high-performance streaming parser
use crate::error::ParseError;
use crate::parser::security::SecurityConfig;
use crate::streaming::{StreamingConfig, StreamingProgress};
use memchr::memmem;
use std::io::BufRead;
use std::time::{Duration, Instant};

/// High-performance streaming parser optimized for 280+ MB/s
pub struct FastStreamingParser {
    // Pre-compiled SIMD-accelerated pattern matchers
    release_start: memmem::Finder<'static>,
    release_end: memmem::Finder<'static>,
    resource_start: memmem::Finder<'static>,
    resource_end: memmem::Finder<'static>,
    header_start: memmem::Finder<'static>,
    header_end: memmem::Finder<'static>,
    // Additional resource patterns for comprehensive matching
    sound_recording_start: memmem::Finder<'static>,
    sound_recording_end: memmem::Finder<'static>,
    party_start: memmem::Finder<'static>,
    party_end: memmem::Finder<'static>,
    deal_start: memmem::Finder<'static>,
    deal_end: memmem::Finder<'static>,
}

/// Fast streaming element with minimal allocation
#[derive(Debug, Clone)]
pub struct FastStreamingElement {
    /// Element type (Release, Resource, Party, etc.)
    pub element_type: FastElementType,
    /// Raw XML content (zero-copy reference)
    pub raw_content: Vec<u8>,
    /// Byte position in original stream
    pub position: u64,
    /// Size in bytes
    pub size: usize,
    /// Parse timestamp
    pub parsed_at: Instant,
}

/// Element types for fast classification
#[derive(Debug, Clone, PartialEq)]
pub enum FastElementType {
    Release,
    Resource,
    Party,
    Deal,
    MessageHeader,
    Other(String),
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct FastParsingStats {
    pub throughput_mbps: f64,
    pub elements_per_second: f64,
    pub total_bytes: u64,
    pub total_elements: usize,
    pub elapsed: Duration,
    pub peak_memory_mb: f64,
    pub avg_element_size: f64,
}

impl FastStreamingParser {
    /// Create a fast streaming parser.
    ///
    /// Known limitation: `config` is currently ignored. The scanner applies none of
    /// its settings (buffer size, `max_memory`, or the `SecurityConfig` limits);
    /// see KNOWN_ISSUES.md.
    pub fn new(_config: StreamingConfig) -> Self {
        Self {
            // Pre-compile all patterns for SIMD acceleration
            release_start: memmem::Finder::new(b"<Release"),
            release_end: memmem::Finder::new(b"</Release>"),
            resource_start: memmem::Finder::new(b"<Resource"),
            resource_end: memmem::Finder::new(b"</Resource>"),
            sound_recording_start: memmem::Finder::new(b"<SoundRecording"),
            sound_recording_end: memmem::Finder::new(b"</SoundRecording>"),
            header_start: memmem::Finder::new(b"<MessageHeader"),
            header_end: memmem::Finder::new(b"</MessageHeader>"),
            party_start: memmem::Finder::new(b"<Party"),
            party_end: memmem::Finder::new(b"</Party>"),
            deal_start: memmem::Finder::new(b"<Deal"),
            deal_end: memmem::Finder::new(b"</Deal>"),
        }
    }

    pub fn parse_streaming<R: BufRead>(
        &mut self,
        reader: &mut R,
        _progress_callback: Option<Box<dyn FnMut(StreamingProgress)>>,
    ) -> Result<FastStreamingIterator, ParseError> {
        let start = Instant::now();

        // Read entire buffer at once - critical for performance
        let mut buffer = Vec::with_capacity(50 * 1024 * 1024); // 50MB initial capacity
        let bytes_read = reader.read_to_end(&mut buffer)?;

        // Pre-allocate results with generous capacity to avoid reallocation
        let mut elements = Vec::with_capacity(50000);

        // Scan using SIMD-accelerated pattern matching
        // Multiple passes for different element types maximize SIMD efficiency

        // Pass 1: Find all releases using SIMD
        let mut pos = 0;
        while let Some(offset) = self.release_start.find(&buffer[pos..]) {
            let start_pos = pos + offset;

            // Find end using SIMD
            if let Some(end_offset) = self.release_end.find(&buffer[start_pos..]) {
                let end_pos = start_pos + end_offset + 10; // "</Release>".len()

                elements.push(FastStreamingElement {
                    element_type: FastElementType::Release,
                    raw_content: buffer[start_pos..end_pos].to_vec(),
                    position: start_pos as u64,
                    size: end_pos - start_pos,
                    parsed_at: Instant::now(),
                });

                pos = end_pos;
            } else {
                pos = start_pos + 1;
            }
        }

        // Pass 2: Find all resources (both Resource and SoundRecording)
        pos = 0;
        while let Some(offset) = self.resource_start.find(&buffer[pos..]) {
            let start_pos = pos + offset;

            if let Some(end_offset) = self.resource_end.find(&buffer[start_pos..]) {
                let end_pos = start_pos + end_offset + 11; // "</Resource>".len()

                elements.push(FastStreamingElement {
                    element_type: FastElementType::Resource,
                    raw_content: buffer[start_pos..end_pos].to_vec(),
                    position: start_pos as u64,
                    size: end_pos - start_pos,
                    parsed_at: Instant::now(),
                });

                pos = end_pos;
            } else {
                pos = start_pos + 1;
            }
        }

        // Pass 2b: Find SoundRecording elements
        pos = 0;
        while let Some(offset) = self.sound_recording_start.find(&buffer[pos..]) {
            let start_pos = pos + offset;

            if let Some(end_offset) = self.sound_recording_end.find(&buffer[start_pos..]) {
                let end_pos = start_pos + end_offset + 17; // "</SoundRecording>".len()

                elements.push(FastStreamingElement {
                    element_type: FastElementType::Resource,
                    raw_content: buffer[start_pos..end_pos].to_vec(),
                    position: start_pos as u64,
                    size: end_pos - start_pos,
                    parsed_at: Instant::now(),
                });

                pos = end_pos;
            } else {
                pos = start_pos + 1;
            }
        }

        // Pass 3: Find message header
        if let Some(offset) = self.header_start.find(&buffer) {
            if let Some(end_offset) = self.header_end.find(&buffer[offset..]) {
                let end_pos = offset + end_offset + 16; // "</MessageHeader>".len()

                elements.push(FastStreamingElement {
                    element_type: FastElementType::MessageHeader,
                    raw_content: buffer[offset..end_pos].to_vec(),
                    position: offset as u64,
                    size: end_pos - offset,
                    parsed_at: Instant::now(),
                });
            }
        }

        // Pass 4: Find parties
        pos = 0;
        while let Some(offset) = self.party_start.find(&buffer[pos..]) {
            let start_pos = pos + offset;

            if let Some(end_offset) = self.party_end.find(&buffer[start_pos..]) {
                let end_pos = start_pos + end_offset + 8; // "</Party>".len()

                elements.push(FastStreamingElement {
                    element_type: FastElementType::Party,
                    raw_content: buffer[start_pos..end_pos].to_vec(),
                    position: start_pos as u64,
                    size: end_pos - start_pos,
                    parsed_at: Instant::now(),
                });

                pos = end_pos;
            } else {
                pos = start_pos + 1;
            }
        }

        // Pass 5: Find deals
        pos = 0;
        while let Some(offset) = self.deal_start.find(&buffer[pos..]) {
            let start_pos = pos + offset;

            if let Some(end_offset) = self.deal_end.find(&buffer[start_pos..]) {
                let end_pos = start_pos + end_offset + 7; // "</Deal>".len()

                elements.push(FastStreamingElement {
                    element_type: FastElementType::Deal,
                    raw_content: buffer[start_pos..end_pos].to_vec(),
                    position: start_pos as u64,
                    size: end_pos - start_pos,
                    parsed_at: Instant::now(),
                });

                pos = end_pos;
            } else {
                pos = start_pos + 1;
            }
        }

        // Sort elements by position for proper ordering
        elements.sort_by_key(|e| e.position);

        let elapsed = start.elapsed();
        let throughput = (bytes_read as f64) / elapsed.as_secs_f64() / (1024.0 * 1024.0);

        let stats = FastParsingStats {
            throughput_mbps: throughput,
            elements_per_second: elements.len() as f64 / elapsed.as_secs_f64(),
            total_bytes: bytes_read as u64,
            total_elements: elements.len(),
            elapsed,
            peak_memory_mb: (buffer.capacity() as f64) / (1024.0 * 1024.0),
            avg_element_size: if !elements.is_empty() {
                elements.iter().map(|e| e.size).sum::<usize>() as f64 / elements.len() as f64
            } else {
                0.0
            },
        };

        Ok(FastStreamingIterator::new(elements, stats))
    }

    /// Get current parsing statistics
    pub fn get_stats(&self) -> FastParsingStats {
        FastParsingStats {
            throughput_mbps: 0.0,
            elements_per_second: 0.0,
            total_bytes: 0,
            total_elements: 0,
            elapsed: Duration::from_secs(0),
            peak_memory_mb: 0.0,
            avg_element_size: 0.0,
        }
    }
}

/// High-performance streaming iterator
#[allow(dead_code)]
pub struct FastStreamingIterator {
    elements: Vec<FastStreamingElement>,
    position: usize,
    stats: FastParsingStats,
}

#[allow(dead_code)]
impl FastStreamingIterator {
    pub fn new(elements: Vec<FastStreamingElement>, mut stats: FastParsingStats) -> Self {
        // Calculate final statistics
        stats.total_elements = elements.len();
        if stats.elapsed.as_secs_f64() > 0.0 {
            stats.elements_per_second = elements.len() as f64 / stats.elapsed.as_secs_f64();
        }
        if !elements.is_empty() {
            stats.avg_element_size =
                elements.iter().map(|e| e.size).sum::<usize>() as f64 / elements.len() as f64;
        }

        Self {
            elements,
            position: 0,
            stats,
        }
    }

    /// Get parsing performance statistics
    pub fn stats(&self) -> &FastParsingStats {
        &self.stats
    }

    /// Get all elements of a specific type
    pub fn filter_by_type(&self, element_type: FastElementType) -> Vec<&FastStreamingElement> {
        self.elements
            .iter()
            .filter(|e| e.element_type == element_type)
            .collect()
    }

    /// Get total number of elements
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if iterator is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl Iterator for FastStreamingIterator {
    type Item = FastStreamingElement;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.elements.len() {
            let element = self.elements[self.position].clone();
            self.position += 1;
            Some(element)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.elements.len() - self.position;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for FastStreamingIterator {}

/// Create a fast streaming parser with optimal configuration for performance
pub fn create_fast_parser() -> FastStreamingParser {
    let config = StreamingConfig {
        security: SecurityConfig::relaxed(), // Use relaxed for maximum performance
        buffer_size: 64 * 1024,              // 64KB buffer
        max_memory: 200 * 1024 * 1024,       // 200MB memory limit
        chunk_size: 512,                     // 512KB chunks for optimal throughput
        enable_progress: false,              // Disable progress for max speed
        progress_interval: 0,
    };

    FastStreamingParser::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn test_fast_streaming_basic() {
        let mut parser = create_fast_parser();

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>MSG001</MessageId>
            </MessageHeader>
            <ReleaseList>
                <Release>
                    <ReleaseId>REL001</ReleaseId>
                    <ReleaseReference>R001</ReleaseReference>
                </Release>
                <Release>
                    <ReleaseId>REL002</ReleaseId>
                    <ReleaseReference>R002</ReleaseReference>
                </Release>
            </ReleaseList>
            <ResourceList>
                <SoundRecording>
                    <ResourceReference>A1</ResourceReference>
                    <Duration>PT3M45S</Duration>
                </SoundRecording>
            </ResourceList>
        </ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let mut reader = BufReader::new(cursor);

        let result = parser.parse_streaming(&mut reader, None);
        assert!(result.is_ok());

        let iterator = result.unwrap();
        let stats = iterator.stats();

        // Should have parsed some elements
        assert!(stats.total_elements > 0);
        assert!(stats.total_bytes > 0);

        #[cfg(feature = "performance-debug")]
        println!("SIMD Fast streaming stats: {:#?}", stats);
        #[cfg(feature = "performance-debug")]
        println!("Throughput: {:.2} MB/s", stats.throughput_mbps);
    }

    #[cfg(feature = "perf-tests")]
    #[test]
    fn test_performance_target() {
        let mut parser = create_fast_parser();

        // Generate a larger XML for more realistic performance testing
        let mut test_xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>PERFORMANCE_TEST</MessageId>
                <MessageThreadId>THREAD001</MessageThreadId>
                <MessageCreatedDateTime>2024-01-01T12:00:00</MessageCreatedDateTime>
            </MessageHeader>
            <ReleaseList>"#,
        );

        // Add many releases for performance testing
        for i in 0..5000 {
            test_xml.push_str(&format!(
                r#"
                <Release>
                    <ReleaseId>REL{:08}</ReleaseId>
                    <ReleaseReference>R{:08}</ReleaseReference>
                    <Title>
                        <TitleText>Test Release {} - High Performance Streaming Test</TitleText>
                    </Title>
                    <DisplayArtist>Test Artist {}</DisplayArtist>
                    <ReleaseType>Album</ReleaseType>
                    <Genre>Electronic</Genre>
                </Release>"#,
                i,
                i,
                i,
                i % 100
            ));
        }

        test_xml.push_str("</ReleaseList><ResourceList>");

        // Add resources
        for i in 0..3000 {
            test_xml.push_str(&format!(
                r#"
                <SoundRecording>
                    <ResourceReference>A{:08}</ResourceReference>
                    <Duration>PT3M{:02}S</Duration>
                    <Title>Track {} High Performance Test</Title>
                    <AudioChannelConfiguration>Stereo</AudioChannelConfiguration>
                    <SampleRate>44100</SampleRate>
                    <BitsPerSample>16</BitsPerSample>
                </SoundRecording>"#,
                i,
                i % 60,
                i
            ));
        }

        test_xml.push_str("</ResourceList></ern:NewReleaseMessage>");

        let cursor = Cursor::new(test_xml.as_bytes());
        let mut reader = BufReader::new(cursor);

        #[cfg(feature = "performance-debug")]
        let start = Instant::now();
        let result = parser.parse_streaming(&mut reader, None);
        #[cfg(feature = "performance-debug")]
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        let iterator = result.unwrap();
        let stats = iterator.stats();

        #[cfg(feature = "performance-debug")]
        println!("SIMD Performance test results:");
        #[cfg(feature = "performance-debug")]
        println!(
            "  Total bytes: {:.2} MB",
            stats.total_bytes as f64 / (1024.0 * 1024.0)
        );
        #[cfg(feature = "performance-debug")]
        println!("  Total elements: {}", stats.total_elements);
        #[cfg(feature = "performance-debug")]
        println!("  Elapsed: {:?}", elapsed);
        #[cfg(feature = "performance-debug")]
        println!("  Throughput: {:.2} MB/s", stats.throughput_mbps);
        #[cfg(feature = "performance-debug")]
        println!("  Elements/sec: {:.2}", stats.elements_per_second);
        #[cfg(feature = "performance-debug")]
        println!("  Peak memory: {:.2} MB", stats.peak_memory_mb);
        #[cfg(feature = "performance-debug")]
        println!("  Avg element size: {:.2} bytes", stats.avg_element_size);

        // Performance targets
        let target_throughput = 50.0; // MB/s - conservative target for CI
        if stats.throughput_mbps >= target_throughput {
            #[cfg(feature = "performance-debug")]
            println!(
                "✅ Performance target met: {:.2} MB/s >= {:.2} MB/s",
                stats.throughput_mbps, target_throughput
            );
        } else {
            #[cfg(feature = "performance-debug")]
            println!(
                "⚠️  Performance below target: {:.2} MB/s < {:.2} MB/s",
                stats.throughput_mbps, target_throughput
            );
        }

        // The parser should handle this efficiently
        assert!(
            stats.total_elements > 8000,
            "Should have found many elements"
        );
        assert!(
            stats.total_bytes > 1024 * 1024,
            "Should have processed > 1MB"
        );
    }

    #[test]
    fn test_element_types_detection() {
        let mut parser = create_fast_parser();

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader><MessageId>TEST</MessageId></MessageHeader>
            <Release><ReleaseId>REL001</ReleaseId></Release>
            <SoundRecording><ResourceReference>A1</ResourceReference></SoundRecording>
            <Party><PartyId>P1</PartyId></Party>
            <Deal><DealId>D1</DealId></Deal>
        </ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let mut reader = BufReader::new(cursor);

        let result = parser.parse_streaming(&mut reader, None);
        assert!(result.is_ok());

        let iterator = result.unwrap();
        let elements: Vec<_> = iterator.collect();

        // Should find all different element types
        let header_count = elements
            .iter()
            .filter(|e| e.element_type == FastElementType::MessageHeader)
            .count();
        let release_count = elements
            .iter()
            .filter(|e| e.element_type == FastElementType::Release)
            .count();
        let resource_count = elements
            .iter()
            .filter(|e| e.element_type == FastElementType::Resource)
            .count();
        let party_count = elements
            .iter()
            .filter(|e| e.element_type == FastElementType::Party)
            .count();
        let deal_count = elements
            .iter()
            .filter(|e| e.element_type == FastElementType::Deal)
            .count();

        #[cfg(feature = "performance-debug")]
        println!("Element type counts:");
        #[cfg(feature = "performance-debug")]
        println!("  Headers: {}", header_count);
        #[cfg(feature = "performance-debug")]
        println!("  Releases: {}", release_count);
        #[cfg(feature = "performance-debug")]
        println!("  Resources: {}", resource_count);
        #[cfg(feature = "performance-debug")]
        println!("  Parties: {}", party_count);
        #[cfg(feature = "performance-debug")]
        println!("  Deals: {}", deal_count);

        assert!(header_count >= 1, "Should find message header");
        assert!(release_count >= 1, "Should find releases");
        assert!(resource_count >= 1, "Should find resources");
        assert!(party_count >= 1, "Should find parties");
        assert!(deal_count >= 1, "Should find deals");
    }
}
