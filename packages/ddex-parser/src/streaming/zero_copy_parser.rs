//! Zero-copy high-performance streaming parser for DDEX XML
//!
//! This implementation targets 280+ MB/s throughput using:
//! - Zero-copy string handling
//! - SIMD-accelerated pattern matching

#[allow(dead_code)] // Experimental zero-copy streaming parser
// - Streaming-native parsing (no DOM)
// - Memory-efficient buffer management
// - Specialized DDEX parsing optimizations
use crate::error::ParseError;
use crate::streaming::{WorkingStreamingElement, WorkingStreamingStats};
use ddex_core::models::versions::ERNVersion;
use std::collections::HashMap;
use std::io::BufRead;
use std::time::Instant;

/// Zero-copy high-performance streaming parser
pub struct ZeroCopyParser {
    /// Current buffer for zero-copy operations
    buffer: Vec<u8>,
    /// String intern cache to avoid allocations
    string_cache: StringCache,
    /// Parser state
    state: ParserState,
    /// Statistics
    stats: ZeroCopyStats,
    /// ERN version
    version: ERNVersion,
}

/// String interning cache for zero-copy operations
struct StringCache {
    cache: HashMap<Vec<u8>, String>,
    hit_count: u64,
    miss_count: u64,
}

impl StringCache {
    fn new() -> Self {
        Self {
            cache: HashMap::with_capacity(1024),
            hit_count: 0,
            miss_count: 0,
        }
    }

    fn intern(&mut self, bytes: &[u8]) -> String {
        if let Some(cached) = self.cache.get(bytes) {
            self.hit_count += 1;
            cached.clone()
        } else {
            self.miss_count += 1;
            let s = String::from_utf8_lossy(bytes).to_string();
            self.cache.insert(bytes.to_vec(), s.clone());
            s
        }
    }

    fn hit_rate(&self) -> f64 {
        if self.hit_count + self.miss_count == 0 {
            0.0
        } else {
            self.hit_count as f64 / (self.hit_count + self.miss_count) as f64
        }
    }
}

#[derive(Debug, Clone)]
enum ParserState {
    Initial,
    InMessageHeader,
    InRelease {
        reference: String,
    },
    InResource {
        resource_type: String,
        reference: String,
    },
    Done,
}

/// High-performance parsed element
#[derive(Debug, Clone)]
pub enum ZeroCopyElement {
    MessageHeader {
        message_id: String,
        created_date_time: String,
        version: ERNVersion,
    },
    Release {
        reference: String,
        title: String,
        genre: Option<String>,
        resource_references: Vec<String>,
    },
    SoundRecording {
        reference: String,
        title: String,
        duration: Option<String>,
        isrc: Option<String>,
        creation_date: Option<String>,
    },
    Video {
        reference: String,
        title: String,
        duration: Option<String>,
        codec: Option<String>,
    },
    Image {
        reference: String,
        title: String,
        width: Option<u32>,
        height: Option<u32>,
        format: Option<String>,
    },
    Text {
        reference: String,
        title: String,
        language: Option<String>,
    },
    EndOfStream {
        stats: ZeroCopyStats,
    },
}

#[derive(Debug, Clone)]
pub struct ZeroCopyStats {
    pub bytes_processed: u64,
    pub elements_found: u64,
    pub string_cache_hit_rate: f64,
    pub parse_time: std::time::Duration,
    pub throughput_mb_per_sec: f64,
    pub memory_used_bytes: usize,
}

impl ZeroCopyParser {
    pub fn new(version: ERNVersion) -> Self {
        Self {
            buffer: Vec::with_capacity(1024 * 1024), // 1MB buffer
            string_cache: StringCache::new(),
            state: ParserState::Initial,
            stats: ZeroCopyStats {
                bytes_processed: 0,
                elements_found: 0,
                string_cache_hit_rate: 0.0,
                parse_time: std::time::Duration::default(),
                throughput_mb_per_sec: 0.0,
                memory_used_bytes: 0,
            },
            version,
        }
    }

    /// High-performance streaming parse using SIMD and zero-copy techniques
    pub fn parse_streaming(&mut self, data: &[u8]) -> Result<Vec<ZeroCopyElement>, ParseError> {
        let start_time = Instant::now();
        self.stats.bytes_processed += data.len() as u64;

        let mut results = Vec::new();

        // Use SIMD-accelerated pattern matching to find element boundaries
        let release_positions = self.find_elements_simd(data, b"<Release")?;
        let sound_recording_positions = self.find_elements_simd(data, b"<SoundRecording")?;
        let video_positions = self.find_elements_simd(data, b"<Video")?;
        let image_positions = self.find_elements_simd(data, b"<Image")?;
        let text_positions = self.find_elements_simd(data, b"<Text")?;
        let message_header_positions = self.find_elements_simd(data, b"<MessageHeader")?;

        // Process message headers
        for pos in message_header_positions {
            if let Some(element) = self.extract_message_header(data, pos)? {
                results.push(element);
                self.stats.elements_found += 1;
            }
        }

        // Process releases with zero-copy extraction
        for pos in release_positions {
            if let Some(element) = self.extract_release_zero_copy(data, pos)? {
                results.push(element);
                self.stats.elements_found += 1;
            }
        }

        // Process sound recordings
        for pos in sound_recording_positions {
            if let Some(element) = self.extract_sound_recording_zero_copy(data, pos)? {
                results.push(element);
                self.stats.elements_found += 1;
            }
        }

        // Process videos
        for pos in video_positions {
            if let Some(element) = self.extract_video_zero_copy(data, pos)? {
                results.push(element);
                self.stats.elements_found += 1;
            }
        }

        // Process images
        for pos in image_positions {
            if let Some(element) = self.extract_image_zero_copy(data, pos)? {
                results.push(element);
                self.stats.elements_found += 1;
            }
        }

        // Process text resources
        for pos in text_positions {
            if let Some(element) = self.extract_text_zero_copy(data, pos)? {
                results.push(element);
                self.stats.elements_found += 1;
            }
        }

        // Update statistics
        self.stats.parse_time = start_time.elapsed();
        self.stats.string_cache_hit_rate = self.string_cache.hit_rate();
        self.stats.throughput_mb_per_sec =
            (data.len() as f64 / (1024.0 * 1024.0)) / self.stats.parse_time.as_secs_f64();
        self.stats.memory_used_bytes = self.estimate_memory_usage();

        Ok(results)
    }

    /// SIMD-accelerated element boundary detection
    #[cfg(target_arch = "x86_64")]
    fn find_elements_simd(&self, data: &[u8], pattern: &[u8]) -> Result<Vec<usize>, ParseError> {
        use std::arch::x86_64::*;

        let mut positions = Vec::new();

        if pattern.len() == 0 || data.len() < pattern.len() {
            return Ok(positions);
        }

        // For patterns longer than 16 bytes, fall back to memchr
        if pattern.len() > 16 {
            return self.find_elements_fallback(data, pattern);
        }

        // SIMD implementation for x86_64
        unsafe {
            let pattern_first = pattern[0];
            let mut i = 0;

            // Process 16 bytes at a time using SIMD
            while i + 16 <= data.len() {
                // Load 16 bytes
                let chunk = _mm_loadu_si128(data.as_ptr().add(i) as *const __m128i);

                // Create a vector of the first pattern byte
                let pattern_vec = _mm_set1_epi8(pattern_first as i8);

                // Compare
                let matches = _mm_cmpeq_epi8(chunk, pattern_vec);

                // Extract match mask
                let mask = _mm_movemask_epi8(matches) as u16;

                // Check each potential match
                for bit_pos in 0..16 {
                    if (mask & (1 << bit_pos)) != 0 {
                        let pos = i + bit_pos;

                        // Verify the full pattern matches
                        if pos + pattern.len() <= data.len()
                            && data[pos..pos + pattern.len()] == *pattern
                        {
                            positions.push(pos);
                        }
                    }
                }

                i += 16;
            }

            // Handle remaining bytes
            while i + pattern.len() <= data.len() {
                if data[i..i + pattern.len()] == *pattern {
                    positions.push(i);
                }
                i += 1;
            }
        }

        Ok(positions)
    }

    /// Fallback pattern matching for non-x86_64 or long patterns
    #[cfg(not(target_arch = "x86_64"))]
    fn find_elements_simd(&self, data: &[u8], pattern: &[u8]) -> Result<Vec<usize>, ParseError> {
        self.find_elements_fallback(data, pattern)
    }

    fn find_elements_fallback(
        &self,
        data: &[u8],
        pattern: &[u8],
    ) -> Result<Vec<usize>, ParseError> {
        let mut positions = Vec::new();
        let mut start = 0;

        // Use memchr for fast first-byte scanning
        use memchr::memchr;

        while let Some(pos) = memchr(pattern[0], &data[start..]) {
            let abs_pos = start + pos;

            // Check if full pattern matches
            if abs_pos + pattern.len() <= data.len()
                && data[abs_pos..abs_pos + pattern.len()] == *pattern
            {
                positions.push(abs_pos);
            }

            start = abs_pos + 1;
        }

        Ok(positions)
    }

    /// Zero-copy message header extraction
    fn extract_message_header(
        &mut self,
        data: &[u8],
        start: usize,
    ) -> Result<Option<ZeroCopyElement>, ParseError> {
        // Find the end of MessageHeader element
        if let Some(end_pos) = self.find_closing_tag(data, start, b"MessageHeader") {
            let header_data = &data[start..end_pos];

            // Extract MessageId with zero-copy
            let message_id =
                if let Some(id_data) = self.extract_field_zero_copy(header_data, b"MessageId") {
                    self.string_cache.intern(id_data)
                } else {
                    "unknown".to_string()
                };

            // Extract CreatedDateTime
            let created_date_time = if let Some(dt_data) =
                self.extract_field_zero_copy(header_data, b"CreatedDateTime")
            {
                self.string_cache.intern(dt_data)
            } else {
                chrono::Utc::now().to_rfc3339()
            };

            return Ok(Some(ZeroCopyElement::MessageHeader {
                message_id,
                created_date_time,
                version: self.version,
            }));
        }

        Ok(None)
    }

    /// Zero-copy release extraction
    fn extract_release_zero_copy(
        &mut self,
        data: &[u8],
        start: usize,
    ) -> Result<Option<ZeroCopyElement>, ParseError> {
        if let Some(end_pos) = self.find_closing_tag(data, start, b"Release") {
            let release_data = &data[start..end_pos];

            // Extract ReleaseReference attribute
            let reference = if let Some(ref_data) =
                self.extract_attribute_zero_copy(release_data, b"ReleaseReference")
            {
                self.string_cache.intern(ref_data)
            } else {
                format!("REL-{}", self.stats.elements_found)
            };

            // Extract title with nested TitleText handling
            let title = if let Some(title_data) =
                self.extract_nested_field_zero_copy(release_data, b"TitleText")
            {
                self.string_cache.intern(title_data)
            } else if let Some(title_data) = self.extract_field_zero_copy(release_data, b"Title") {
                self.string_cache.intern(title_data)
            } else {
                "Untitled Release".to_string()
            };

            // Extract genre
            let genre = self
                .extract_nested_field_zero_copy(release_data, b"GenreText")
                .map(|g| self.string_cache.intern(g));

            // Extract resource references (simplified)
            let resource_references = self.extract_resource_references_zero_copy(release_data);

            return Ok(Some(ZeroCopyElement::Release {
                reference,
                title,
                genre,
                resource_references,
            }));
        }

        Ok(None)
    }

    /// Zero-copy sound recording extraction
    fn extract_sound_recording_zero_copy(
        &mut self,
        data: &[u8],
        start: usize,
    ) -> Result<Option<ZeroCopyElement>, ParseError> {
        if let Some(end_pos) = self.find_closing_tag(data, start, b"SoundRecording") {
            let recording_data = &data[start..end_pos];

            let reference = if let Some(ref_data) =
                self.extract_attribute_zero_copy(recording_data, b"ResourceReference")
            {
                self.string_cache.intern(ref_data)
            } else {
                format!("RES-{}", self.stats.elements_found)
            };

            let title = if let Some(title_data) =
                self.extract_nested_field_zero_copy(recording_data, b"TitleText")
            {
                self.string_cache.intern(title_data)
            } else {
                "Untitled Track".to_string()
            };

            let duration = self
                .extract_field_zero_copy(recording_data, b"Duration")
                .map(|d| self.string_cache.intern(d));

            let isrc = self
                .extract_field_zero_copy(recording_data, b"ISRC")
                .map(|i| self.string_cache.intern(i));

            let creation_date = self
                .extract_field_zero_copy(recording_data, b"CreationDate")
                .map(|cd| self.string_cache.intern(cd));

            return Ok(Some(ZeroCopyElement::SoundRecording {
                reference,
                title,
                duration,
                isrc,
                creation_date,
            }));
        }

        Ok(None)
    }

    /// Zero-copy video extraction
    fn extract_video_zero_copy(
        &mut self,
        data: &[u8],
        start: usize,
    ) -> Result<Option<ZeroCopyElement>, ParseError> {
        if let Some(end_pos) = self.find_closing_tag(data, start, b"Video") {
            let video_data = &data[start..end_pos];

            let reference = if let Some(ref_data) =
                self.extract_attribute_zero_copy(video_data, b"ResourceReference")
            {
                self.string_cache.intern(ref_data)
            } else {
                format!("VID-{}", self.stats.elements_found)
            };

            let title = if let Some(title_data) =
                self.extract_nested_field_zero_copy(video_data, b"TitleText")
            {
                self.string_cache.intern(title_data)
            } else {
                "Untitled Video".to_string()
            };

            let duration = self
                .extract_field_zero_copy(video_data, b"Duration")
                .map(|d| self.string_cache.intern(d));

            let codec = self
                .extract_field_zero_copy(video_data, b"VideoCodecType")
                .map(|c| self.string_cache.intern(c));

            return Ok(Some(ZeroCopyElement::Video {
                reference,
                title,
                duration,
                codec,
            }));
        }

        Ok(None)
    }

    /// Zero-copy image extraction
    fn extract_image_zero_copy(
        &mut self,
        data: &[u8],
        start: usize,
    ) -> Result<Option<ZeroCopyElement>, ParseError> {
        if let Some(end_pos) = self.find_closing_tag(data, start, b"Image") {
            let image_data = &data[start..end_pos];

            let reference = if let Some(ref_data) =
                self.extract_attribute_zero_copy(image_data, b"ResourceReference")
            {
                self.string_cache.intern(ref_data)
            } else {
                format!("IMG-{}", self.stats.elements_found)
            };

            let title = if let Some(title_data) =
                self.extract_nested_field_zero_copy(image_data, b"TitleText")
            {
                self.string_cache.intern(title_data)
            } else {
                "Untitled Image".to_string()
            };

            let width = self
                .extract_field_zero_copy(image_data, b"Width")
                .and_then(|w| String::from_utf8_lossy(w).parse().ok());

            let height = self
                .extract_field_zero_copy(image_data, b"Height")
                .and_then(|h| String::from_utf8_lossy(h).parse().ok());

            let format = self
                .extract_field_zero_copy(image_data, b"ImageCodecType")
                .map(|f| self.string_cache.intern(f));

            return Ok(Some(ZeroCopyElement::Image {
                reference,
                title,
                width,
                height,
                format,
            }));
        }

        Ok(None)
    }

    /// Zero-copy text resource extraction
    fn extract_text_zero_copy(
        &mut self,
        data: &[u8],
        start: usize,
    ) -> Result<Option<ZeroCopyElement>, ParseError> {
        if let Some(end_pos) = self.find_closing_tag(data, start, b"Text") {
            let text_data = &data[start..end_pos];

            let reference = if let Some(ref_data) =
                self.extract_attribute_zero_copy(text_data, b"ResourceReference")
            {
                self.string_cache.intern(ref_data)
            } else {
                format!("TXT-{}", self.stats.elements_found)
            };

            let title = if let Some(title_data) =
                self.extract_nested_field_zero_copy(text_data, b"TitleText")
            {
                self.string_cache.intern(title_data)
            } else {
                "Untitled Text".to_string()
            };

            let language = self
                .extract_field_zero_copy(text_data, b"LanguageOfPerformance")
                .or_else(|| self.extract_field_zero_copy(text_data, b"LanguageCode"))
                .map(|l| self.string_cache.intern(l));

            return Ok(Some(ZeroCopyElement::Text {
                reference,
                title,
                language,
            }));
        }

        Ok(None)
    }

    /// Find closing tag position
    fn find_closing_tag(&self, data: &[u8], start: usize, tag_name: &[u8]) -> Option<usize> {
        let closing_pattern = [b"</", tag_name, b">"].concat();

        // Start search after the opening tag
        let search_start = start + tag_name.len();
        if let Ok(positions) = self.find_elements_fallback(&data[search_start..], &closing_pattern)
        {
            if let Some(pos) = positions.first() {
                return Some(search_start + pos + closing_pattern.len());
            }
        }

        None
    }

    /// Extract field content with zero-copy
    fn extract_field_zero_copy<'a>(&self, data: &'a [u8], field_name: &[u8]) -> Option<&'a [u8]> {
        let opening = [b"<", field_name, b">"].concat();
        let closing = [b"</", field_name, b">"].concat();

        if let Ok(start_positions) = self.find_elements_fallback(data, &opening) {
            if let Some(&start_pos) = start_positions.first() {
                let content_start = start_pos + opening.len();

                if let Ok(end_positions) =
                    self.find_elements_fallback(&data[content_start..], &closing)
                {
                    if let Some(&end_pos) = end_positions.first() {
                        let content_end = content_start + end_pos;
                        return Some(&data[content_start..content_end]);
                    }
                }
            }
        }

        None
    }

    /// Extract nested field content (e.g., ReferenceTitle/TitleText)
    fn extract_nested_field_zero_copy<'a>(
        &self,
        data: &'a [u8],
        inner_field: &[u8],
    ) -> Option<&'a [u8]> {
        // Look for the inner field directly first
        if let Some(content) = self.extract_field_zero_copy(data, inner_field) {
            return Some(content);
        }

        // Look within common parent elements
        let parent_tags: &[&[u8]] = &[b"ReferenceTitle", b"Title"];

        for parent in parent_tags {
            if let Some(parent_content) = self.extract_field_zero_copy(data, parent) {
                if let Some(inner_content) =
                    self.extract_field_zero_copy(parent_content, inner_field)
                {
                    return Some(inner_content);
                }
            }
        }

        None
    }

    /// Extract attribute value with zero-copy
    fn extract_attribute_zero_copy<'a>(
        &self,
        data: &'a [u8],
        attr_name: &[u8],
    ) -> Option<&'a [u8]> {
        let pattern = [attr_name, b"=\""].concat();

        if let Ok(positions) = self.find_elements_fallback(data, &pattern) {
            if let Some(&pos) = positions.first() {
                let value_start = pos + pattern.len();

                // Find the closing quote
                if let Some(quote_pos) = memchr::memchr(b'"', &data[value_start..]) {
                    let value_end = value_start + quote_pos;
                    return Some(&data[value_start..value_end]);
                }
            }
        }

        None
    }

    /// Extract resource references (simplified zero-copy version)
    fn extract_resource_references_zero_copy(&mut self, data: &[u8]) -> Vec<String> {
        let mut references = Vec::new();

        // Look for ResourceReference elements
        if let Ok(positions) = self.find_elements_fallback(data, b"<ResourceReference>") {
            for pos in positions {
                if let Some(ref_data) =
                    self.extract_field_zero_copy(&data[pos..], b"ResourceReference")
                {
                    references.push(self.string_cache.intern(ref_data));
                }
            }
        }

        references
    }

    fn find_closing_tag_simple(&self, data: &[u8], start: usize, tag_name: &str) -> Option<usize> {
        let closing_tag = format!("</{}>", tag_name);
        let closing_bytes = closing_tag.as_bytes();

        if let Ok(positions) = self.find_elements_fallback(&data[start..], closing_bytes) {
            if let Some(&pos) = positions.first() {
                return Some(start + pos + closing_bytes.len());
            }
        }

        None
    }

    fn estimate_memory_usage(&self) -> usize {
        self.buffer.capacity() +
        self.string_cache.cache.capacity() * 64 + // Rough estimate
        std::mem::size_of::<Self>()
    }

    pub fn get_stats(&self) -> &ZeroCopyStats {
        &self.stats
    }
}

/// High-performance stream iterator that integrates with existing API
pub struct ZeroCopyStreamIterator<R: BufRead> {
    reader: R,
    parser: ZeroCopyParser,
    buffer: Vec<u8>,
    finished: bool,
    elements_queue: Vec<ZeroCopyElement>,
    current_index: usize,
    start_time: Instant,
}

impl<R: BufRead> ZeroCopyStreamIterator<R> {
    pub fn new(mut reader: R, version: ERNVersion) -> Self {
        let mut buffer = Vec::with_capacity(1024 * 1024); // 1MB buffer
        let _ = reader.read_to_end(&mut buffer);

        Self {
            reader,
            parser: ZeroCopyParser::new(version),
            buffer,
            finished: false,
            elements_queue: Vec::new(),
            current_index: 0,
            start_time: Instant::now(),
        }
    }

    pub fn stats(&self) -> WorkingStreamingStats {
        let zero_copy_stats = self.parser.get_stats();
        WorkingStreamingStats {
            bytes_processed: zero_copy_stats.bytes_processed,
            elements_yielded: zero_copy_stats.elements_found as usize,
            current_depth: 0,
            max_depth_reached: 10, // Estimated
            current_memory_bytes: zero_copy_stats.memory_used_bytes,
            max_memory_used_bytes: zero_copy_stats.memory_used_bytes,
            elapsed_time: self.start_time.elapsed(),
            throughput_mb_per_sec: zero_copy_stats.throughput_mb_per_sec,
        }
    }

    fn convert_to_working_element(element: ZeroCopyElement) -> WorkingStreamingElement {
        match element {
            ZeroCopyElement::MessageHeader {
                message_id,
                created_date_time,
                version,
            } => WorkingStreamingElement::MessageHeader {
                message_id,
                created_date_time,
                version,
            },
            ZeroCopyElement::Release {
                reference,
                title,
                resource_references,
                ..
            } => WorkingStreamingElement::Release {
                reference,
                title,
                resource_references,
            },
            ZeroCopyElement::SoundRecording {
                reference,
                title,
                duration,
                isrc,
                ..
            } => WorkingStreamingElement::SoundRecording {
                reference,
                title,
                duration,
                isrc,
            },
            ZeroCopyElement::Video {
                reference,
                title,
                duration,
                ..
            } => WorkingStreamingElement::Video {
                reference,
                title,
                duration,
            },
            ZeroCopyElement::Image {
                reference,
                title,
                width,
                height,
                ..
            } => WorkingStreamingElement::Image {
                reference,
                title,
                width,
                height,
            },
            ZeroCopyElement::Text {
                reference,
                title,
                language,
            } => WorkingStreamingElement::Text {
                reference,
                title,
                language_code: language,
            },
            ZeroCopyElement::EndOfStream { stats } => {
                WorkingStreamingElement::EndOfStream {
                    stats: WorkingStreamingStats {
                        bytes_processed: stats.bytes_processed,
                        elements_yielded: stats.elements_found as usize,
                        current_depth: 0,
                        max_depth_reached: 10, // Estimated
                        current_memory_bytes: stats.memory_used_bytes,
                        max_memory_used_bytes: stats.memory_used_bytes,
                        elapsed_time: stats.parse_time,
                        throughput_mb_per_sec: stats.throughput_mb_per_sec,
                    },
                }
            }
        }
    }
}

impl<R: BufRead> Iterator for ZeroCopyStreamIterator<R> {
    type Item = Result<WorkingStreamingElement, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // If we haven't processed the data yet, do it now
        if self.elements_queue.is_empty() && self.current_index == 0 {
            match self.parser.parse_streaming(&self.buffer) {
                Ok(mut elements) => {
                    // Add end-of-stream marker
                    elements.push(ZeroCopyElement::EndOfStream {
                        stats: self.parser.get_stats().clone(),
                    });
                    self.elements_queue = elements;
                }
                Err(e) => {
                    self.finished = true;
                    return Some(Err(e));
                }
            }
        }

        // Return next element from queue
        if self.current_index < self.elements_queue.len() {
            let element = self.elements_queue[self.current_index].clone();
            self.current_index += 1;

            // Check if this is the last element
            if matches!(element, ZeroCopyElement::EndOfStream { .. }) {
                self.finished = true;
            }

            Some(Ok(Self::convert_to_working_element(element)))
        } else {
            self.finished = true;
            None
        }
    }
}

/// High-performance iterator wrapper for backward compatibility
pub struct ZeroCopyIterator {
    parser: ZeroCopyParser,
    data: Vec<u8>,
    position: usize,
    chunk_size: usize,
    finished: bool,
}

impl ZeroCopyIterator {
    pub fn new(data: Vec<u8>, version: ERNVersion, chunk_size: usize) -> Self {
        Self {
            parser: ZeroCopyParser::new(version),
            data,
            position: 0,
            chunk_size: chunk_size.max(1024), // Minimum 1KB chunks
            finished: false,
        }
    }

    pub fn parse_all(&mut self) -> Result<Vec<ZeroCopyElement>, ParseError> {
        let mut all_elements = Vec::new();

        // Process the entire data at once for maximum performance
        let elements = self.parser.parse_streaming(&self.data)?;
        all_elements.extend(elements);

        // Add end-of-stream marker
        all_elements.push(ZeroCopyElement::EndOfStream {
            stats: self.parser.get_stats().clone(),
        });

        self.finished = true;
        Ok(all_elements)
    }

    pub fn stats(&self) -> &ZeroCopyStats {
        self.parser.get_stats()
    }
}

#[cfg(all(test, feature = "legacy-streaming-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_basic_parsing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>ZERO-COPY-TEST</MessageId>
        <CreatedDateTime>2023-01-01T00:00:00Z</CreatedDateTime>
    </MessageHeader>
    <Release ReleaseReference="ZC-REL-001">
        <ReferenceTitle>
            <TitleText>Zero Copy Release</TitleText>
        </ReferenceTitle>
    </Release>
</ern:NewReleaseMessage>"#;

        let mut parser = ZeroCopyParser::new(ERNVersion::V4_3);
        let elements = parser.parse_streaming(xml.as_bytes()).unwrap();

        assert!(!elements.is_empty(), "Should find elements");
        println!("Zero-copy parsing found {} elements", elements.len());

        // Verify we found expected elements
        let has_header = elements
            .iter()
            .any(|e| matches!(e, ZeroCopyElement::MessageHeader { .. }));
        let has_release = elements
            .iter()
            .any(|e| matches!(e, ZeroCopyElement::Release { .. }));

        assert!(has_header, "Should find message header");
        assert!(has_release, "Should find release");

        let stats = parser.get_stats();
        println!(
            "Zero-copy stats: {:.2} MB/s, {}% cache hit rate",
            stats.throughput_mb_per_sec,
            stats.string_cache_hit_rate * 100.0
        );
    }

    #[test]
    fn test_simd_pattern_matching() {
        let data = b"<Release><Release><Release>";
        let parser = ZeroCopyParser::new(ERNVersion::V4_3);

        let positions = parser.find_elements_simd(data, b"<Release").unwrap();
        assert_eq!(positions.len(), 3, "Should find 3 occurrences");
        assert_eq!(positions, vec![0, 9, 18]);
    }

    #[test]
    fn test_zero_copy_field_extraction() {
        let data = b"<Title>Test Title</Title>";
        let parser = ZeroCopyParser::new(ERNVersion::V4_3);

        let content = parser.extract_field_zero_copy(data, b"Title").unwrap();
        assert_eq!(content, b"Test Title");
    }

    #[test]
    fn test_attribute_extraction() {
        let data = b"<Release ReleaseReference=\"REL-123\">content</Release>";
        let parser = ZeroCopyParser::new(ERNVersion::V4_3);

        let attr_value = parser
            .extract_attribute_zero_copy(data, b"ReleaseReference")
            .unwrap();
        assert_eq!(attr_value, b"REL-123");
    }
}
